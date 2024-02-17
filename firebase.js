// This script initializes firebase.
//
// Your web app's Firebase configuration
// For Firebase JS SDK v7.20.0 and later, measurementId is optional

// Compat modules for firebase ui
import "https://www.gstatic.com/firebasejs/10.8.0/firebase-analytics-compat.js";
import "https://www.gstatic.com/firebasejs/10.8.0/firebase-app-compat.js";
import "https://www.gstatic.com/firebasejs/10.8.0/firebase-auth-compat.js";
import "https://www.gstatic.com/firebasejs/10.8.0/firebase-functions-compat.js";
import "https://www.gstatic.com/firebasejs/10.8.0/firebase-performance-compat.js";
import "https://www.gstatic.com/firebasejs/10.8.0/firebase-storage-compat.js";
import "https://www.gstatic.com/firebasejs/ui/6.1.0/firebase-ui-auth.js";

const firebaseConfig = {
  apiKey: "AIzaSyBCtkQM2P7eBAFtKWnlGfyTNyTHE8y5wXY",
  authDomain: "homotopy-io.firebaseapp.com",
  databaseURL: "https://homotopy-io.firebaseio.com",
  projectId: "homotopy-io",
  storageBucket: "homotopy-io.appspot.com",
  messagingSenderId: "872831343483",
  appId: "1:872831343483:web:4de585eb01b14a27ad3bde",
  measurementId: "G-CYGQVSSM6Q"
};

// Initialize Firebase
console.log("Loading firebase...");

// Initialize Firebase
const app = app.initializeApp(firebaseConfig);
const analytics = app.analytics();
const perf = app.performance();

const auth = app.auth();

const storage = app.storage();
const functions = app.functions('us-east1');

if (location.hostname === "localhost") {
  console.log("localhost detected, using firebase emulators");
  auth.useEmulator("http://127.0.0.1:9099");
  storage.useEmulator("127.0.0.1", 9199);
  functions.useEmulator("127.0.0.1", 5001);
}

const ui = new firebaseui.auth.AuthUI(auth);
