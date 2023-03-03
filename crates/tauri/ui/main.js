const { invoke } = window.__TAURI__.tauri;

async function getUserinfos() {
   greetMsgEl.textContent = await invoke("get_userinfos", { name: greetInputEl.value });
}

const userInfosViewModel = {
    propriétés: ko.observableArray( [
      // { propriété: 'name', valeur : 'LOL' },
    ]),

    fournisseur: ko.observable("Microsoft"),

    clicFournisseur : function() {
        this.propriétés.removeAll();
        return true;
    },

    enableUserInfos: ko.observable(true),

    erreurFetch: ko.observable(""),

    getUserInfos: async function() {
        this.enableUserInfos(false);
        this.erreurFetch("");

        const headers = new Headers({
            'Content-Type': 'application/json',
        });

        const csrfCookie =
            document.cookie
            .split(';')
            .find((item) => item.trim().startsWith('Csrf-Token='));

        if (csrfCookie) {
            headers.set('X-Csrf-Token', csrfCookie.split('=')[1])
        }

        const request = new Request('/userinfos', {
            method: 'POST',
            headers: headers,
            cache: 'no-cache',
            redirect: 'error',
            body: '{ "fournisseur": "' + this.fournisseur() + '", "origine": "' + location.origin + '" }'
        });

        fetch(request)
        .then(response => response.json())
        .then(data => {
            if (data.hasOwnProperty("redirectOP")) {
                sessionStorage.setItem("actionAfterAuth", "getUserInfos");
                window.location.replace(data.redirectOP);
            } else {
                this.propriétés.removeAll();
                for (const propriété of data) {
                    this.propriétés.push(propriété)
                }
                this.enableUserInfos(true);
            }
        })
        .catch((error) => {
            console.log("Erreur Fetch: " + error);
            this.erreurFetch(error);
            this.enableUserInfos(true);
        });
    }
}

ko.applyBindings(userInfosViewModel);
