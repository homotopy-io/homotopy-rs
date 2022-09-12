// This file is included in `bindings.rs`

import { getAuth, GoogleAuthProvider, signInWithPopup, signOut } from "https://www.gstatic.com/firebasejs/9.7.0/firebase-auth.js";

const auth = getAuth();
const provider = new GoogleAuthProvider();

// TODO: return a proper data structure
export function logIn() {

  signInWithPopup(auth, provider)
    .then((result) => {
      // This gives you a Google Access Token. You can use it to access the Google API.
      const credential = GoogleAuthProvider.credentialFromResult(result);
      const token = credential.accessToken;
      // The signed-in user info.
      const user = result.user;

      document.getElementById("username").innerHTML = user.displayName;
      console.log(user.displayName);
      // ...
    }).catch((error) => {
      // Handle Errors here.
      const errorCode = error.code;
      const errorMessage = error.message;
      // The email of the user's account used.
      const email = error.customData.email;
      // The AuthCredential type that was used.
      const credential = GoogleAuthProvider.credentialFromError(error);
      
      console.log(errorMessage);
    });

}

export function logOut() {
  signOut(auth).then(() => {
    document.getElementById("username").innerHTML = "Guest user";
    // Sign-out successful.
  }).catch((error) => {
      console.log(error);
  });
}

