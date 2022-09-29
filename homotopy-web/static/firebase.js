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

console.log("Loading firebase...");
window.firebase_working = false;

// TODO: Add SDKs for Firebase products that you want to use
// https://firebase.google.com/docs/web/setup#available-libraries

// Compat modules for firebase ui
await import("https://www.gstatic.com/firebasejs/9.7.0/firebase-app-compat.js")
	.catch(err => console.error("Failed to import firebase-app-compat", err))
	.then(mod => {
		// Initialize Firebase
		const app = firebase.initializeApp(firebaseConfig);

		import("https://www.gstatic.com/firebasejs/9.7.0/firebase-analytics-compat.js")
		.catch(err => console.error("Failed to import firebase-analytics-compat", err))
		.then(mod => {
			const analytics = firebase.analytics();

			import("https://www.gstatic.com/firebasejs/9.7.0/firebase-performance-compat.js")
			.catch(err => console.error("Failed to import firebase-performance-compat", err))
			.then(mod => {
				const perf = firebase.performance();

				import("https://www.gstatic.com/firebasejs/9.7.0/firebase-auth-compat.js")
				.catch(err => console.error("Failed to import firebase-auth-compat", err))
				.then(mod => { const auth = firebase.auth(); window.firebase_working = true; })
			})
		})
	});
