use std::sync::Arc;

use accesskit::{DefaultActionVerb, Role};
use masonry::app_driver::{AppDriver, DriverCtx};
use masonry::vello::Scene;
use masonry::widget::{Align, CrossAxisAlignment, Flex, Label, RootWidget, SizedBox, WidgetRef};
use masonry::{
    AccessCtx, AccessEvent, Action, BoxConstraints, Color, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Point, PointerEvent, Size,
    StatusChange, TextEvent, Widget, WidgetId, WidgetPod,
};
use smallvec::{smallvec, SmallVec};
use tracing::{trace, trace_span, Span};
use winit::dpi::LogicalSize;
use winit::window::Window;

mod table;
use serde_json::value::Value;
use std::{fmt, thread};
use table::{Table, TableData};

mod pkce;
use pkce::Pkce;


const FINISH_GET_USERINFOS: Selector<Result<Vec<Vec<String>>, String>> = Selector::new("finish_get_userinfos");
const ID_MS: &str = include_str!("../../../secrets/clientid.microsoft");
const SECRET_MS: &str = include_str!("../../../secrets/secret.microsoft");
const ID_GG: &str = include_str!("../../../secrets/clientid.google");
const SECRET_GG: &str = include_str!("../../../secrets/secret.google");
const AUTH_MS: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const AUTH_GG: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_MS: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const TOKEN_GG: &str = "https://oauth2.googleapis.com/token";
const INFOS_MS: &str = "https://graph.microsoft.com/oidc/userinfo";
const INFOS_GG: &str = "https://openidconnect.googleapis.com/v1/userinfo";

#[dynamic]
static mut TOKEN: Option<(Fournisseur, Pkce)> = None;

#[derive(Clone)]
struct AppData {
    radio_fournisseur: Fournisseur,
    label_fournisseur: String,
    infos: Arc<TableData>,
    en_traitement: bool,
    erreur: String,
}

#[derive(Clone, PartialEq)]
pub enum Fournisseur {
    Microsoft,
    Google,
}

impl fmt::Display for Fournisseur {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fournisseur = match self {
            Fournisseur::Microsoft => "Microsoft",
            Fournisseur::Google => "Google",
        };
        f.write_str(fournisseur)
    }
}

impl Fournisseur {
    fn endpoints(&self) -> (&str, &str) {
        match self {
            Self::Microsoft => (AUTH_MS, TOKEN_MS),
            Self::Google => (AUTH_GG, TOKEN_GG),
        }
    }

    fn secrets(&self) -> (&str, &str) {
        match self {
            Self::Microsoft => (ID_MS, SECRET_MS),
            Self::Google => (ID_GG, SECRET_GG),
        }
    }

    fn userinfos(&self) -> &str {
        match self {
            Self::Microsoft => INFOS_MS,
            Self::Google => INFOS_GG,
        }
    }
}

fn request_userinfos(f: &Fournisseur) -> Result<Value, anyhow::Error> {
    let token = TOKEN.read();
    if token.is_some() {
        let (fournisseur, secret) = token.as_ref().unwrap();
        if f != fournisseur || secret.is_expired() {
            drop(token);
            TOKEN.write().replace((f.to_owned(), Pkce::new(f)?));
        }
    } else {
        drop(token);
        TOKEN.write().replace((f.to_owned(), Pkce::new(f)?));
    }

    Ok(ureq::get(f.userinfos())
        .set("Authorization", &format!("Bearer {}", TOKEN.read().as_ref().unwrap().1.secret()))
        .call()?
        .into_json::<Value>()?)
}

fn get_userinfos(sink: ExtEventSink, fournisseur: Fournisseur) {
    thread::spawn(move || {
        let result = match request_userinfos(&fournisseur) {
            Ok(value) => match value {
                Value::Object(map) => {
                    let table: Vec<Vec<String>> = map.iter().map(|(k, v)| vec![k.to_owned(), v.to_string().replace('"', "")]).collect();
                    Ok(table)
                }
                _ => Err("La valeur doit être un map".to_string()),
            },
            Err(e) => Err(e.to_string()),
        };

        sink.submit_command(FINISH_GET_USERINFOS, result, Target::Auto)
            .expect("command failed to submit");
    });
}

struct Delegate;

impl AppDelegate<AppData> for Delegate {
    fn command(&mut self, _ctx: &mut DelegateCtx, _target: Target, cmd: &Command, data: &mut AppData, _env: &Env) -> Handled {
        match cmd.get(FINISH_GET_USERINFOS) {
            Some(Ok(infos)) => {
                data.en_traitement = false;
                data.infos = Arc::new(TableData {
                    rows: infos.to_owned(),
                    header: vec!["Propriété".to_owned(), "Valeur".to_owned()],
                });
                Handled::Yes
            }
            Some(Err(e)) => {
                data.en_traitement = false;
                data.erreur = e.to_owned();
                Handled::Yes
            }
            None => Handled::No,
        }
    }
}

fn ui_builder() -> impl Widget {
    let mut oidc = Flex::column()
        .must_fill_main_axis(true)
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .main_axis_alignment(MainAxisAlignment::Center);

    let png_data = ImageBuf::from_data(include_bytes!("openid-icon-100x100.png")).unwrap();
    oidc.add_child(Image::new(png_data));
    oidc.add_child(Label::new("OpenID Connect").with_text_size(25.));
    oidc.add_default_spacer();

    oidc.add_child(Label::new("Fournisseur:"));
    oidc.add_default_spacer();
    let mut fournisseurs = Vector::new();
    fournisseurs.push_back((Fournisseur::Microsoft.to_string(), Fournisseur::Microsoft));
    fournisseurs.push_back((Fournisseur::Google.to_string(), Fournisseur::Google));
    oidc.add_child(RadioGroup::row(fournisseurs).lens(AppData::radio_fournisseur));
    oidc.add_default_spacer();

    let bouton = Button::new("UserInfos")
        .on_click(|ctx, data: &mut AppData, _| {
            data.erreur = String::new();
            data.label_fournisseur = data.radio_fournisseur.to_string();
            data.en_traitement = true;
            get_userinfos(ctx.get_external_handle(), data.radio_fournisseur.clone());
        })
        .fix_height(30.0);

    oidc.add_child(Either::new(|data, _env| data.en_traitement, Spinner::new(), bouton));

    let infos = Flex::column()
        .must_fill_main_axis(true)
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .main_axis_alignment(MainAxisAlignment::Center)
        .with_child(
            Label::new(|data: &AppData, _env: &_| format!("UserInfos {}", data.label_fournisseur))
                .with_text_size(18.)
                .with_text_color(Color::from_hex_str("FFA500").unwrap()),
        )
        .with_default_spacer()
        .with_child(
            Table::new()
                .with_header_text_color(Color::from_hex_str("FFA500").unwrap())
                .lens(AppData::infos),
        );

    let main = Flex::row().with_default_spacer().with_child(oidc).with_spacer(40.).with_child(infos);

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(main)
        .with_default_spacer()
        .with_child(
            Flex::row().with_default_spacer().with_child(
                Label::new(|data: &AppData, _env: &_| data.erreur.clone())
                    .with_text_color(Color::rgb(1., 0., 0.))
                    .expand_width(),
            ),
        )
    //    .debug_paint_layout()
}



#[derive(Clone)]
struct CalcState {
    /// The number displayed. Generally a valid float.
    value: String,
    operand: f64,
    operator: char,
    in_num: bool,
}

#[derive(Clone, Copy)]
enum CalcAction {
    Digit(u8),
    Op(char),
}

struct CalcButton {
    inner: WidgetPod<SizedBox>,
    action: CalcAction,
    base_color: Color,
    active_color: Color,
}

// ---

impl CalcState {
    fn digit(&mut self, digit: u8) {
        if !self.in_num {
            self.value.clear();
            self.in_num = true;
        }
        let ch = (b'0' + digit) as char;
        self.value.push(ch);
    }

    fn display(&mut self) {
        self.value = self.operand.to_string();
    }

    fn compute(&mut self) {
        if self.in_num {
            let operand2 = self.value.parse().unwrap_or(0.0);
            let result = match self.operator {
                '+' => Some(self.operand + operand2),
                '−' => Some(self.operand - operand2),
                '×' => Some(self.operand * operand2),
                '÷' => Some(self.operand / operand2),
                _ => None,
            };
            if let Some(result) = result {
                self.operand = result;
                self.display();
                self.in_num = false;
            }
        }
    }

    fn op(&mut self, op: char) {
        match op {
            '+' | '−' | '×' | '÷' | '=' => {
                self.compute();
                self.operand = self.value.parse().unwrap_or(0.0);
                self.operator = op;
                self.in_num = false;
            }
            '±' => {
                if self.in_num {
                    if self.value.starts_with('−') {
                        self.value = self.value[3..].to_string();
                    } else {
                        self.value = ["−", &self.value].concat();
                    }
                } else {
                    self.operand = -self.operand;
                    self.display();
                }
            }
            '.' => {
                if !self.in_num {
                    self.value = "0".to_string();
                    self.in_num = true;
                }
                if self.value.find('.').is_none() {
                    self.value.push('.');
                }
            }
            'c' => {
                self.value = "0".to_string();
                self.in_num = false;
            }
            'C' => {
                self.value = "0".to_string();
                self.operator = 'C';
                self.in_num = false;
            }
            '⌫' => {
                if self.in_num {
                    self.value.pop();
                    if self.value.is_empty() || self.value == "−" {
                        self.value = "0".to_string();
                        self.in_num = false;
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

impl CalcButton {
    fn new(inner: SizedBox, action: CalcAction, base_color: Color, active_color: Color) -> Self {
        Self {
            inner: WidgetPod::new(inner),
            action,
            base_color,
            active_color,
        }
    }
}

impl Widget for CalcButton {
    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) {
        match event {
            PointerEvent::PointerDown(_, _) => {
                if !ctx.is_disabled() {
                    ctx.get_mut(&mut self.inner).set_background(self.active_color);
                    ctx.set_active(true);
                    ctx.request_paint();
                    trace!("CalcButton {:?} pressed", ctx.widget_id());
                }
            }
            PointerEvent::PointerUp(_, _) => {
                if ctx.is_active() && !ctx.is_disabled() {
                    ctx.submit_action(Action::Other(Arc::new(self.action)));
                    ctx.request_paint();
                    trace!("CalcButton {:?} released", ctx.widget_id());
                }
                ctx.get_mut(&mut self.inner).set_background(self.base_color);
                ctx.set_active(false);
            }
            _ => (),
        }
        self.inner.on_pointer_event(ctx, event);
    }

    fn on_text_event(&mut self, ctx: &mut EventCtx, event: &TextEvent) {
        self.inner.on_text_event(ctx, event);
    }

    fn on_access_event(&mut self, ctx: &mut EventCtx, event: &AccessEvent) {
        if event.target == ctx.widget_id() {
            match event.action {
                accesskit::Action::Default => {
                    ctx.submit_action(Action::Other(Arc::new(self.action)));
                    ctx.request_paint();
                }
                _ => {}
            }
        }
        ctx.skip_child(&mut self.inner);
    }

    fn on_status_change(&mut self, ctx: &mut LifeCycleCtx, event: &StatusChange) {
        match event {
            StatusChange::HotChanged(true) => {
                ctx.get_mut(&mut self.inner).set_border(Color::WHITE, 3.0);
                ctx.request_paint();
            }
            StatusChange::HotChanged(false) => {
                ctx.get_mut(&mut self.inner).set_border(Color::TRANSPARENT, 3.0);
                ctx.request_paint();
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle) {
        self.inner.lifecycle(ctx, event);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints) -> Size {
        let size = self.inner.layout(ctx, bc);
        ctx.place_child(&mut self.inner, Point::ORIGIN);

        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        self.inner.paint(ctx, scene);
    }

    fn accessibility_role(&self) -> Role {
        Role::Button
    }

    fn accessibility(&mut self, ctx: &mut AccessCtx) {
        let _name = match self.action {
            CalcAction::Digit(digit) => digit.to_string(),
            CalcAction::Op(op) => op.to_string(),
        };
        // We may want to add a name if it doesn't interfere with the child label
        // ctx.current_node().set_name(name);
        ctx.current_node().set_default_action_verb(DefaultActionVerb::Click);

        self.inner.accessibility(ctx);
    }

    fn children(&self) -> SmallVec<[WidgetRef<'_, dyn Widget>; 16]> {
        smallvec![self.inner.as_dyn()]
    }

    fn make_trace_span(&self) -> Span {
        trace_span!("CalcButton")
    }
}

impl AppDriver for CalcState {
    fn on_action(&mut self, ctx: &mut DriverCtx<'_>, _widget_id: WidgetId, action: Action) {
        match action {
            Action::Other(payload) => match payload.downcast_ref::<CalcAction>().unwrap() {
                CalcAction::Digit(digit) => self.digit(*digit),
                CalcAction::Op(op) => self.op(*op),
            },
            _ => unreachable!(),
        }

        ctx.get_root::<RootWidget<Flex>>()
            .get_element()
            .child_mut(1)
            .unwrap()
            .downcast::<Label>()
            .set_text(&*self.value);
    }
}

// ---

fn op_button_with_label(op: char, label: String) -> CalcButton {
    const BLUE: Color = Color::rgb8(0x00, 0x8d, 0xdd);
    const LIGHT_BLUE: Color = Color::rgb8(0x5c, 0xc4, 0xff);

    CalcButton::new(
        SizedBox::new(Align::centered(Label::new(label).with_text_size(24.)))
            .background(BLUE)
            .expand(),
        CalcAction::Op(op),
        BLUE,
        LIGHT_BLUE,
    )
}

fn op_button(op: char) -> CalcButton {
    op_button_with_label(op, op.to_string())
}

fn digit_button(digit: u8) -> CalcButton {
    const GRAY: Color = Color::rgb8(0x3a, 0x3a, 0x3a);
    const LIGHT_GRAY: Color = Color::rgb8(0x71, 0x71, 0x71);
    CalcButton::new(
        SizedBox::new(Align::centered(Label::new(format!("{digit}")).with_text_size(24.)))
            .background(GRAY)
            .expand(),
        CalcAction::Digit(digit),
        GRAY,
        LIGHT_GRAY,
    )
}

fn flex_row(w1: impl Widget + 'static, w2: impl Widget + 'static, w3: impl Widget + 'static, w4: impl Widget + 'static) -> impl Widget {
    Flex::row()
        .with_flex_child(w1, 1.0)
        .with_spacer(1.0)
        .with_flex_child(w2, 1.0)
        .with_spacer(1.0)
        .with_flex_child(w3, 1.0)
        .with_spacer(1.0)
        .with_flex_child(w4, 1.0)
}

fn build_calc() -> impl Widget {
    let display = Label::new(String::new()).with_text_size(32.0);
    Flex::column()
        .with_flex_spacer(0.2)
        .with_child(display)
        .with_flex_spacer(0.2)
        .cross_axis_alignment(CrossAxisAlignment::End)
        .with_flex_child(
            flex_row(
                op_button_with_label('c', "CE".to_string()),
                op_button('C'),
                op_button('⌫'),
                op_button('÷'),
            ),
            1.0,
        )
        .with_spacer(1.0)
        .with_flex_child(flex_row(digit_button(7), digit_button(8), digit_button(9), op_button('×')), 1.0)
        .with_spacer(1.0)
        .with_flex_child(flex_row(digit_button(4), digit_button(5), digit_button(6), op_button('−')), 1.0)
        .with_spacer(1.0)
        .with_flex_child(flex_row(digit_button(1), digit_button(2), digit_button(3), op_button('+')), 1.0)
        .with_spacer(1.0)
        .with_flex_child(flex_row(op_button('±'), digit_button(0), op_button('.'), op_button('=')), 1.0)
}

pub fn main() {
    let window_size = LogicalSize::new(223., 300.);

    let window_attributes = Window::default_attributes()
        .with_title("Simple Calculator")
        .with_resizable(true)
        .with_min_inner_size(window_size);

    let calc_state = CalcState {
        value: "0".to_string(),
        operand: 0.0,
        operator: 'C',
        in_num: false,
    };

    masonry::event_loop_runner::run(window_attributes, RootWidget::new(build_calc()), calc_state).unwrap();
}
