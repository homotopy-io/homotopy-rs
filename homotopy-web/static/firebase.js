// This script initializes firebase.
//
// Your web app's Firebase configuration
// For Firebase JS SDK v7.20.0 and later, measurementId is optional
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

// Compat modules for firebase ui
await import("https://www.gstatic.com/firebasejs/9.15.0/firebase-app-compat.js")
    .catch(err => {
        console.error("Failed to import firebase-app-compat", err);
        throw err;
    });
// Initialize Firebase
const app = firebase.initializeApp(firebaseConfig);

await import("https://www.gstatic.com/firebasejs/9.15.0/firebase-analytics-compat.js")
    .catch(err => {
        console.error("Failed to import firebase-analytics-compat", err);
        throw err;
    });
const analytics = firebase.analytics();

await import("https://www.gstatic.com/firebasejs/9.15.0/firebase-performance-compat.js")
    .catch(err => {
        console.error("Failed to import firebase-performance-compat", err);
        throw err;
    });
const perf = firebase.performance();
