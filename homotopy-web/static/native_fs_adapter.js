
import { showSaveFilePicker } from 'https://cdn.jsdelivr.net/npm/native-file-system-adapter/mod.js';

window.download = function(name, blob, ext, mime) {
	new Promise(() => showSaveFilePicker({
	  _preferPolyfill: false,
	  suggestedName: name,
	  types: [
		{ accept: { mime: [ ext ] } },
	  ],
	  excludeAcceptAllOption: false
	})
	.then(fileHandle => fileHandle.createWritable())
	.then(writer => {writer.write(blob).then(() => writer.close())}), undefined);
}
window.gen_download = function(name, blob, ext, mime) {
	return (() => window.download(name, blob, ext, mime));
}
