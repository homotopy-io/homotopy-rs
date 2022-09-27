# Firebase cloud functions

This folder contains the source code of our Firebase cloud functions. These functions are running on Google's cloud platform as a server-less backend. 
The cloud functions are sitting in `index.js`. They can be called directly via HTTP requests or triggered by background actions like `onCreate`. Use `cargo make serve-firebase` to serve homotopy.io and run firebase emulator locally on your device.

## Developing

To use `firebase`, make sure that you have `node` and `npm` installed.
It is the best to develop and test on a local Firebase emulator. 

Setup the emulator with `firebase init emulators`,
then choose the Firebase features you want to use (e.g. Hosting, Functions, Auth, Firestore, ...).
Some features might require you to install Java.

Start the emulator with `firebase emulators:start`. 

If the emulator UI fails immediately with exit code 1, try downgrading your firebase to version 10.9.2 
(see [this thread](https://stackoverflow.com/questions/72313155/firebase-emulator-fails-at-startup-cannot-find-module-dns-result-order-ipv4fir)).
Find your Firebase directory with `which firebase`. Delete that directory. 
Then, reinstall Firebase with `npm i -g firebase-tools@10.9.2`.
Restart `nix` for the changes to be effective.

## Deploying

You need to login before deploying Firebase functions. Enter `firebase login` then follow the instructions. 
Then, if your account have the permission, deploy with `firebase deploy --only functions`.
