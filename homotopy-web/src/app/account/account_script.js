// This file is included in `bindings.rs`

import "https://www.gstatic.com/firebasejs/9.7.0/firebase-app-compat.js";
import "https://www.gstatic.com/firebasejs/9.7.0/firebase-auth-compat.js";
import "https://www.gstatic.com/firebasejs/9.7.0/firebase-functions-compat.js";

const auth = firebase.auth();
var ui = new firebaseui.auth.AuthUI(auth);

const saveProjectFn = firebase.functions().httpsCallable('saveProject');
const getUserProjectsFn = firebase.functions().httpsCallable('getUserProjects');

export const saveProject = saveProjectFn;
export const getUserProjects = (uid, callback) => {
    return getUserProjectsFn(uid)
        .then(result => callback(result.data.projects));
}

export function initializeUI() {
    //Initialize Firebase UI
    ui.start('#firebaseui-auth-container', {
        signInOptions: [
            // List of OAuth providers supported.
            firebase.auth.EmailAuthProvider.PROVIDER_ID,
            firebase.auth.GoogleAuthProvider.PROVIDER_ID,
            firebase.auth.GithubAuthProvider.PROVIDER_ID
        ],
        signInFlow: "popup",
        callbacks: {
            signInSuccessWithAuthResult: function(authResult, redirectUrl) {
                return false;
            },
        }
        // Other config options...
    });
}

export function resgisterAuthCallback(logInCallback, unsubscribe) {
    if (unsubscribe) {
        unsubscribe();
    }

    return auth.onAuthStateChanged(function(user) {
        if (user) {
            // User is signed in.
            logInCallback({
                uid: user.uid,
                display_name: user.display_name,
                email: user.email,
                photo_url: user.photo_url
            });
        } else {
            // User is signed out.
        }
    }, function(error) {
        console.error(error);
    });
}

export function logOut(logOutCallback) {
    auth.signOut().then(() => {
        // Sign-out successful.
        logOutCallback();
    }).catch((error) => {
        // An error happened.
        console.error(error);
    });
}
