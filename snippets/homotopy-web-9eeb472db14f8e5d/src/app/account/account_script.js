// This file is included in `bindings.rs`

import "https://www.gstatic.com/firebasejs/9.7.0/firebase-app-compat.js";
import "https://www.gstatic.com/firebasejs/9.7.0/firebase-auth-compat.js";

const auth = firebase.auth();
var ui = new firebaseui.auth.AuthUI(auth);

export function initializeUI() {
  //Initialize Firebase UI
  ui.start('#firebaseui-auth-container', {
    signInOptions: [
      // List of OAuth providers supported.
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
      logInCallback(user.displayName);
      
      // Other useful data
      // var displayName = user.displayName;
      // var email = user.email;
      // var emailVerified = user.emailVerified;
      // var photoURL = user.photoURL;
      // var uid = user.uid;
      // var phoneNumber = user.phoneNumber;
      // var providerData = user.providerData;
    } else {
      // User is signed out.
    }
  }, function(error) {
      console.log(error);
  });
}

export function logOut(logOutCallback) {
  auth.signOut().then(() => {
    // Sign-out successful.
    logOutCallback();
  }).catch((error) => {
    // An error happened.
    console.log(error);
  });
}
