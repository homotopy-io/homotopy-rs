
var createHighsModule = (() => {
  var _scriptDir = import.meta.url;
  
  return (
function(createHighsModule) {
  createHighsModule = createHighsModule || {};


var Module;Module||(Module=typeof createHighsModule !== 'undefined' ? createHighsModule : {});var aa,ba;Module.ready=new Promise(function(a,b){aa=a;ba=b});var ca=Object.assign({},Module),da="./this.program",ea="object"==typeof window,h="function"==typeof importScripts,l="object"==typeof process&&"object"==typeof process.versions&&"string"==typeof process.versions.node,v="",fa,w,y,fs,ha,ia;
if(l)v=h?require("path").dirname(v)+"/":__dirname+"/",ia=()=>{ha||(fs=require("fs"),ha=require("path"))},fa=function(a,b){ia();a=ha.normalize(a);return fs.readFileSync(a,b?void 0:"utf8")},y=a=>{a=fa(a,!0);a.buffer||(a=new Uint8Array(a));return a},w=(a,b,c)=>{ia();a=ha.normalize(a);fs.readFile(a,function(d,e){d?c(d):b(e.buffer)})},1<process.argv.length&&(da=process.argv[1].replace(/\\/g,"/")),process.argv.slice(2),process.on("uncaughtException",function(a){throw a;}),process.on("unhandledRejection",
function(a){throw a;}),Module.inspect=function(){return"[Emscripten Module object]"};else if(ea||h)h?v=self.location.href:"undefined"!=typeof document&&document.currentScript&&(v=document.currentScript.src),_scriptDir&&(v=_scriptDir),0!==v.indexOf("blob:")?v=v.substr(0,v.replace(/[?#].*/,"").lastIndexOf("/")+1):v="",fa=a=>{var b=new XMLHttpRequest;b.open("GET",a,!1);b.send(null);return b.responseText},h&&(y=a=>{var b=new XMLHttpRequest;b.open("GET",a,!1);b.responseType="arraybuffer";b.send(null);
return new Uint8Array(b.response)}),w=(a,b,c)=>{var d=new XMLHttpRequest;d.open("GET",a,!0);d.responseType="arraybuffer";d.onload=()=>{200==d.status||0==d.status&&d.response?b(d.response):c()};d.onerror=c;d.send(null)};var ja=Module.print||console.log.bind(console),z=Module.printErr||console.warn.bind(console);Object.assign(Module,ca);ca=null;Module.thisProgram&&(da=Module.thisProgram);var A;Module.wasmBinary&&(A=Module.wasmBinary);var noExitRuntime=Module.noExitRuntime||!0;
"object"!=typeof WebAssembly&&B("no native wasm support detected");var ka,la=!1,ma="undefined"!=typeof TextDecoder?new TextDecoder("utf8"):void 0;
function D(a,b){for(var c=b+NaN,d=b;a[d]&&!(d>=c);)++d;if(16<d-b&&a.buffer&&ma)return ma.decode(a.subarray(b,d));for(c="";b<d;){var e=a[b++];if(e&128){var g=a[b++]&63;if(192==(e&224))c+=String.fromCharCode((e&31)<<6|g);else{var k=a[b++]&63;e=224==(e&240)?(e&15)<<12|g<<6|k:(e&7)<<18|g<<12|k<<6|a[b++]&63;65536>e?c+=String.fromCharCode(e):(e-=65536,c+=String.fromCharCode(55296|e>>10,56320|e&1023))}}else c+=String.fromCharCode(e)}return c}
function na(a,b,c,d){if(!(0<d))return 0;var e=c;d=c+d-1;for(var g=0;g<a.length;++g){var k=a.charCodeAt(g);if(55296<=k&&57343>=k){var t=a.charCodeAt(++g);k=65536+((k&1023)<<10)|t&1023}if(127>=k){if(c>=d)break;b[c++]=k}else{if(2047>=k){if(c+1>=d)break;b[c++]=192|k>>6}else{if(65535>=k){if(c+2>=d)break;b[c++]=224|k>>12}else{if(c+3>=d)break;b[c++]=240|k>>18;b[c++]=128|k>>12&63}b[c++]=128|k>>6&63}b[c++]=128|k&63}}b[c]=0;return c-e}var oa,E,F,pa,G,H;
function qa(){var a=ka.buffer;oa=a;Module.HEAP8=E=new Int8Array(a);Module.HEAP16=pa=new Int16Array(a);Module.HEAP32=G=new Int32Array(a);Module.HEAPU8=F=new Uint8Array(a);Module.HEAPU16=new Uint16Array(a);Module.HEAPU32=H=new Uint32Array(a);Module.HEAPF32=new Float32Array(a);Module.HEAPF64=new Float64Array(a)}var ra=[],sa=[],ta=[];function ua(){var a=Module.preRun.shift();ra.unshift(a)}var I=0,va=null,K=null;
function B(a){if(Module.onAbort)Module.onAbort(a);a="Aborted("+a+")";z(a);la=!0;a=new WebAssembly.RuntimeError(a+". Build with -sASSERTIONS for more info.");ba(a);throw a;}function wa(){return L.startsWith("data:application/octet-stream;base64,")}var L;if(Module.locateFile){if(L="highs.wasm",!wa()){var xa=L;L=Module.locateFile?Module.locateFile(xa,v):v+xa}}else L=(new URL("highs.wasm",import.meta.url)).toString();
function ya(){var a=L;try{if(a==L&&A)return new Uint8Array(A);if(y)return y(a);throw"both async and sync fetching of the wasm failed";}catch(b){B(b)}}
function za(){if(!A&&(ea||h)){if("function"==typeof fetch&&!L.startsWith("file://"))return fetch(L,{credentials:"same-origin"}).then(function(a){if(!a.ok)throw"failed to load wasm binary file at '"+L+"'";return a.arrayBuffer()}).catch(function(){return ya()});if(w)return new Promise(function(a,b){w(L,function(c){a(new Uint8Array(c))},b)})}return Promise.resolve().then(function(){return ya()})}var M,Aa;function Ba(a){for(;0<a.length;)a.shift()(Module)}
function Ca(a){this.ba=a-24;this.Da=function(b){H[this.ba+4>>2]=b};this.Aa=function(b){H[this.ba+8>>2]=b};this.Ba=function(){G[this.ba>>2]=0};this.za=function(){E[this.ba+12>>0]=0};this.Ca=function(){E[this.ba+13>>0]=0};this.fa=function(b,c){this.ya();this.Da(b);this.Aa(c);this.Ba();this.za();this.Ca()};this.ya=function(){H[this.ba+16>>2]=0}}
var Da=0,Ea=(a,b)=>{for(var c=0,d=a.length-1;0<=d;d--){var e=a[d];"."===e?a.splice(d,1):".."===e?(a.splice(d,1),c++):c&&(a.splice(d,1),c--)}if(b)for(;c;c--)a.unshift("..");return a},N=a=>{var b="/"===a.charAt(0),c="/"===a.substr(-1);(a=Ea(a.split("/").filter(d=>!!d),!b).join("/"))||b||(a=".");a&&c&&(a+="/");return(b?"/":"")+a},Fa=a=>{var b=/^(\/?|)([\s\S]*?)((?:\.{1,2}|[^\/]+?|)(\.[^.\/]*|))(?:[\/]*)$/.exec(a).slice(1);a=b[0];b=b[1];if(!a&&!b)return".";b&&(b=b.substr(0,b.length-1));return a+b},Ga=
a=>{if("/"===a)return"/";a=N(a);a=a.replace(/\/$/,"");var b=a.lastIndexOf("/");return-1===b?a:a.substr(b+1)};function Ha(){if("object"==typeof crypto&&"function"==typeof crypto.getRandomValues){var a=new Uint8Array(1);return()=>{crypto.getRandomValues(a);return a[0]}}if(l)try{var b=require("crypto");return()=>b.randomBytes(1)[0]}catch(c){}return()=>B("randomDevice")}
function Ia(){for(var a="",b=!1,c=arguments.length-1;-1<=c&&!b;c--){b=0<=c?arguments[c]:"/";if("string"!=typeof b)throw new TypeError("Arguments to path.resolve must be strings");if(!b)return"";a=b+"/"+a;b="/"===b.charAt(0)}a=Ea(a.split("/").filter(d=>!!d),!b).join("/");return(b?"/":"")+a||"."}function Ja(a,b){for(var c=0,d=0;d<a.length;++d){var e=a.charCodeAt(d);127>=e?c++:2047>=e?c+=2:55296<=e&&57343>=e?(c+=4,++d):c+=3}c=Array(c+1);a=na(a,c,0,c.length);b&&(c.length=a);return c}var Ka=[];
function La(a,b){Ka[a]={input:[],output:[],ga:b};Ma(a,Na)}
var Na={open:function(a){var b=Ka[a.node.rdev];if(!b)throw new O(43);a.tty=b;a.seekable=!1},close:function(a){a.tty.ga.flush(a.tty)},flush:function(a){a.tty.ga.flush(a.tty)},read:function(a,b,c,d){if(!a.tty||!a.tty.ga.ua)throw new O(60);for(var e=0,g=0;g<d;g++){try{var k=a.tty.ga.ua(a.tty)}catch(t){throw new O(29);}if(void 0===k&&0===e)throw new O(6);if(null===k||void 0===k)break;e++;b[c+g]=k}e&&(a.node.timestamp=Date.now());return e},write:function(a,b,c,d){if(!a.tty||!a.tty.ga.ma)throw new O(60);
try{for(var e=0;e<d;e++)a.tty.ga.ma(a.tty,b[c+e])}catch(g){throw new O(29);}d&&(a.node.timestamp=Date.now());return e}},Oa={ua:function(a){if(!a.input.length){var b=null;if(l){var c=Buffer.alloc(256),d=0;try{d=fs.readSync(process.stdin.fd,c,0,256,-1)}catch(e){if(e.toString().includes("EOF"))d=0;else throw e;}0<d?b=c.slice(0,d).toString("utf-8"):b=null}else"undefined"!=typeof window&&"function"==typeof window.prompt?(b=window.prompt("Input: "),null!==b&&(b+="\n")):"function"==typeof readline&&(b=readline(),
null!==b&&(b+="\n"));if(!b)return null;a.input=Ja(b,!0)}return a.input.shift()},ma:function(a,b){null===b||10===b?(ja(D(a.output,0)),a.output=[]):0!=b&&a.output.push(b)},flush:function(a){a.output&&0<a.output.length&&(ja(D(a.output,0)),a.output=[])}},Pa={ma:function(a,b){null===b||10===b?(z(D(a.output,0)),a.output=[]):0!=b&&a.output.push(b)},flush:function(a){a.output&&0<a.output.length&&(z(D(a.output,0)),a.output=[])}},P={V:null,Y:function(){return P.createNode(null,"/",16895,0)},createNode:function(a,
b,c,d){if(24576===(c&61440)||4096===(c&61440))throw new O(63);P.V||(P.V={dir:{node:{Z:P.S.Z,W:P.S.W,lookup:P.S.lookup,ha:P.S.ha,rename:P.S.rename,unlink:P.S.unlink,rmdir:P.S.rmdir,readdir:P.S.readdir,symlink:P.S.symlink},stream:{aa:P.T.aa}},file:{node:{Z:P.S.Z,W:P.S.W},stream:{aa:P.T.aa,read:P.T.read,write:P.T.write,pa:P.T.pa,va:P.T.va,xa:P.T.xa}},link:{node:{Z:P.S.Z,W:P.S.W,readlink:P.S.readlink},stream:{}},qa:{node:{Z:P.S.Z,W:P.S.W},stream:Qa}});c=Ra(a,b,c,d);16384===(c.mode&61440)?(c.S=P.V.dir.node,
c.T=P.V.dir.stream,c.R={}):32768===(c.mode&61440)?(c.S=P.V.file.node,c.T=P.V.file.stream,c.U=0,c.R=null):40960===(c.mode&61440)?(c.S=P.V.link.node,c.T=P.V.link.stream):8192===(c.mode&61440)&&(c.S=P.V.qa.node,c.T=P.V.qa.stream);c.timestamp=Date.now();a&&(a.R[b]=c,a.timestamp=c.timestamp);return c},Qa:function(a){return a.R?a.R.subarray?a.R.subarray(0,a.U):new Uint8Array(a.R):new Uint8Array(0)},ra:function(a,b){var c=a.R?a.R.length:0;c>=b||(b=Math.max(b,c*(1048576>c?2:1.125)>>>0),0!=c&&(b=Math.max(b,
256)),c=a.R,a.R=new Uint8Array(b),0<a.U&&a.R.set(c.subarray(0,a.U),0))},Ia:function(a,b){if(a.U!=b)if(0==b)a.R=null,a.U=0;else{var c=a.R;a.R=new Uint8Array(b);c&&a.R.set(c.subarray(0,Math.min(b,a.U)));a.U=b}},S:{Z:function(a){var b={};b.dev=8192===(a.mode&61440)?a.id:1;b.ino=a.id;b.mode=a.mode;b.nlink=1;b.uid=0;b.gid=0;b.rdev=a.rdev;16384===(a.mode&61440)?b.size=4096:32768===(a.mode&61440)?b.size=a.U:40960===(a.mode&61440)?b.size=a.link.length:b.size=0;b.atime=new Date(a.timestamp);b.mtime=new Date(a.timestamp);
b.ctime=new Date(a.timestamp);b.Ea=4096;b.blocks=Math.ceil(b.size/b.Ea);return b},W:function(a,b){void 0!==b.mode&&(a.mode=b.mode);void 0!==b.timestamp&&(a.timestamp=b.timestamp);void 0!==b.size&&P.Ia(a,b.size)},lookup:function(){throw Sa[44];},ha:function(a,b,c,d){return P.createNode(a,b,c,d)},rename:function(a,b,c){if(16384===(a.mode&61440)){try{var d=Ta(b,c)}catch(g){}if(d)for(var e in d.R)throw new O(55);}delete a.parent.R[a.name];a.parent.timestamp=Date.now();a.name=c;b.R[c]=a;b.timestamp=a.parent.timestamp;
a.parent=b},unlink:function(a,b){delete a.R[b];a.timestamp=Date.now()},rmdir:function(a,b){var c=Ta(a,b),d;for(d in c.R)throw new O(55);delete a.R[b];a.timestamp=Date.now()},readdir:function(a){var b=[".",".."],c;for(c in a.R)a.R.hasOwnProperty(c)&&b.push(c);return b},symlink:function(a,b,c){a=P.createNode(a,b,41471,0);a.link=c;return a},readlink:function(a){if(40960!==(a.mode&61440))throw new O(28);return a.link}},T:{read:function(a,b,c,d,e){var g=a.node.R;if(e>=a.node.U)return 0;a=Math.min(a.node.U-
e,d);if(8<a&&g.subarray)b.set(g.subarray(e,e+a),c);else for(d=0;d<a;d++)b[c+d]=g[e+d];return a},write:function(a,b,c,d,e,g){b.buffer===E.buffer&&(g=!1);if(!d)return 0;a=a.node;a.timestamp=Date.now();if(b.subarray&&(!a.R||a.R.subarray)){if(g)return a.R=b.subarray(c,c+d),a.U=d;if(0===a.U&&0===e)return a.R=b.slice(c,c+d),a.U=d;if(e+d<=a.U)return a.R.set(b.subarray(c,c+d),e),d}P.ra(a,e+d);if(a.R.subarray&&b.subarray)a.R.set(b.subarray(c,c+d),e);else for(g=0;g<d;g++)a.R[e+g]=b[c+g];a.U=Math.max(a.U,e+
d);return d},aa:function(a,b,c){1===c?b+=a.position:2===c&&32768===(a.node.mode&61440)&&(b+=a.node.U);if(0>b)throw new O(28);return b},pa:function(a,b,c){P.ra(a.node,b+c);a.node.U=Math.max(a.node.U,b+c)},va:function(a,b,c,d,e){if(32768!==(a.node.mode&61440))throw new O(43);a=a.node.R;if(e&2||a.buffer!==oa){if(0<c||c+b<a.length)a.subarray?a=a.subarray(c,c+b):a=Array.prototype.slice.call(a,c,c+b);c=!0;B();b=void 0;if(!b)throw new O(48);E.set(a,b)}else c=!1,b=a.byteOffset;return{ba:b,Pa:c}},xa:function(a,
b,c,d,e){if(32768!==(a.node.mode&61440))throw new O(43);if(e&2)return 0;P.T.write(a,b,0,d,c,!1);return 0}}},Ua=null,Va={},Q=[],Wa=1,R=null,Xa=!0,O=null,Sa={},S=(a,b={})=>{a=Ia("/",a);if(!a)return{path:"",node:null};b=Object.assign({ta:!0,na:0},b);if(8<b.na)throw new O(32);a=Ea(a.split("/").filter(k=>!!k),!1);for(var c=Ua,d="/",e=0;e<a.length;e++){var g=e===a.length-1;if(g&&b.parent)break;c=Ta(c,a[e]);d=N(d+"/"+a[e]);c.ia&&(!g||g&&b.ta)&&(c=c.ia.root);if(!g||b.sa)for(g=0;40960===(c.mode&61440);)if(c=
Ya(d),d=Ia(Fa(d),c),c=S(d,{na:b.na+1}).node,40<g++)throw new O(32);}return{path:d,node:c}},Za=a=>{for(var b;;){if(a===a.parent)return a=a.Y.wa,b?"/"!==a[a.length-1]?a+"/"+b:a+b:a;b=b?a.name+"/"+b:a.name;a=a.parent}},$a=(a,b)=>{for(var c=0,d=0;d<b.length;d++)c=(c<<5)-c+b.charCodeAt(d)|0;return(a+c>>>0)%R.length},Ta=(a,b)=>{var c;if(c=(c=ab(a,"x"))?c:a.S.lookup?0:2)throw new O(c,a);for(c=R[$a(a.id,b)];c;c=c.Ha){var d=c.name;if(c.parent.id===a.id&&d===b)return c}return a.S.lookup(a,b)},Ra=(a,b,c,d)=>
{a=new bb(a,b,c,d);b=$a(a.parent.id,a.name);a.Ha=R[b];return R[b]=a},cb={r:0,"r+":2,w:577,"w+":578,a:1089,"a+":1090},db=a=>{var b=["r","w","rw"][a&3];a&512&&(b+="w");return b},ab=(a,b)=>{if(Xa)return 0;if(!b.includes("r")||a.mode&292){if(b.includes("w")&&!(a.mode&146)||b.includes("x")&&!(a.mode&73))return 2}else return 2;return 0},eb=(a,b)=>{try{return Ta(a,b),20}catch(c){}return ab(a,"wx")},fb=(a=0)=>{for(;4096>=a;a++)if(!Q[a])return a;throw new O(33);},gb=(a,b)=>{T||(T=function(){this.fa={}},T.prototype=
{},Object.defineProperties(T.prototype,{object:{get:function(){return this.node},set:function(c){this.node=c}},flags:{get:function(){return this.fa.flags},set:function(c){this.fa.flags=c}},position:{get:function(){return this.fa.position},set:function(c){this.fa.position=c}}}));a=Object.assign(new T,a);b=fb(b);a.fd=b;return Q[b]=a},Qa={open:a=>{a.T=Va[a.node.rdev].T;a.T.open&&a.T.open(a)},aa:()=>{throw new O(70);}},Ma=(a,b)=>{Va[a]={T:b}},hb=(a,b)=>{var c="/"===b,d=!b;if(c&&Ua)throw new O(10);if(!c&&
!d){var e=S(b,{ta:!1});b=e.path;e=e.node;if(e.ia)throw new O(10);if(16384!==(e.mode&61440))throw new O(54);}b={type:a,Ra:{},wa:b,Ga:[]};a=a.Y(b);a.Y=b;b.root=a;c?Ua=a:e&&(e.ia=b,e.Y&&e.Y.Ga.push(b))},U=(a,b,c)=>{var d=S(a,{parent:!0}).node;a=Ga(a);if(!a||"."===a||".."===a)throw new O(28);var e=eb(d,a);if(e)throw new O(e);if(!d.S.ha)throw new O(63);return d.S.ha(d,a,b,c)},ib=(a,b,c)=>{"undefined"==typeof c&&(c=b,b=438);U(a,b|8192,c)},jb=(a,b)=>{if(!Ia(a))throw new O(44);var c=S(b,{parent:!0}).node;
if(!c)throw new O(44);b=Ga(b);var d=eb(c,b);if(d)throw new O(d);if(!c.S.symlink)throw new O(63);c.S.symlink(c,b,a)},Ya=a=>{a=S(a).node;if(!a)throw new O(44);if(!a.S.readlink)throw new O(28);return Ia(Za(a.parent),a.S.readlink(a))},lb=(a,b,c)=>{if(""===a)throw new O(44);if("string"==typeof b){var d=cb[b];if("undefined"==typeof d)throw Error("Unknown file open mode: "+b);b=d}c=b&64?("undefined"==typeof c?438:c)&4095|32768:0;if("object"==typeof a)var e=a;else{a=N(a);try{e=S(a,{sa:!(b&131072)}).node}catch(g){}}d=
!1;if(b&64)if(e){if(b&128)throw new O(20);}else e=U(a,c,0),d=!0;if(!e)throw new O(44);8192===(e.mode&61440)&&(b&=-513);if(b&65536&&16384!==(e.mode&61440))throw new O(54);if(!d&&(c=e?40960===(e.mode&61440)?32:16384===(e.mode&61440)&&("r"!==db(b)||b&512)?31:ab(e,db(b)):44))throw new O(c);if(b&512&&!d){c=e;c="string"==typeof c?S(c,{sa:!0}).node:c;if(!c.S.W)throw new O(63);if(16384===(c.mode&61440))throw new O(31);if(32768!==(c.mode&61440))throw new O(28);if(d=ab(c,"w"))throw new O(d);c.S.W(c,{size:0,
timestamp:Date.now()})}b&=-131713;e=gb({node:e,path:Za(e),flags:b,seekable:!0,position:0,T:e.T,Oa:[],error:!1});e.T.open&&e.T.open(e);!Module.logReadFiles||b&1||(kb||(kb={}),a in kb||(kb[a]=1));return e},mb=(a,b,c)=>{if(null===a.fd)throw new O(8);if(!a.seekable||!a.T.aa)throw new O(70);if(0!=c&&1!=c&&2!=c)throw new O(28);a.position=a.T.aa(a,b,c);a.Oa=[]},nb=()=>{O||(O=function(a,b){this.node=b;this.Ja=function(c){this.$=c};this.Ja(a);this.message="FS error"},O.prototype=Error(),O.prototype.constructor=
O,[44].forEach(a=>{Sa[a]=new O(a);Sa[a].stack="<generic error, no stack>"}))},ob,pb=(a,b)=>{var c=0;a&&(c|=365);b&&(c|=146);return c},V=(a,b,c)=>{a=N("/dev/"+a);var d=pb(!!b,!!c);qb||(qb=64);var e=qb++<<8|0;Ma(e,{open:g=>{g.seekable=!1},close:()=>{c&&c.buffer&&c.buffer.length&&c(10)},read:(g,k,t,m)=>{for(var n=0,r=0;r<m;r++){try{var u=b()}catch(J){throw new O(29);}if(void 0===u&&0===n)throw new O(6);if(null===u||void 0===u)break;n++;k[t+r]=u}n&&(g.node.timestamp=Date.now());return n},write:(g,k,t,
m)=>{for(var n=0;n<m;n++)try{c(k[t+n])}catch(r){throw new O(29);}m&&(g.node.timestamp=Date.now());return n}});ib(a,d,e)},qb,W={},T,kb,X=void 0;function Y(){X+=4;return G[X-4>>2]}function Z(a){a=Q[a];if(!a)throw new O(8);return a}var rb;rb=l?()=>{var a=process.hrtime();return 1E3*a[0]+a[1]/1E6}:()=>performance.now();var sb={};
function tb(){if(!ub){var a={USER:"web_user",LOGNAME:"web_user",PATH:"/",PWD:"/",HOME:"/home/web_user",LANG:("object"==typeof navigator&&navigator.languages&&navigator.languages[0]||"C").replace("-","_")+".UTF-8",_:da||"./this.program"},b;for(b in sb)void 0===sb[b]?delete a[b]:a[b]=sb[b];var c=[];for(b in a)c.push(b+"="+a[b]);ub=c}return ub}var ub;function vb(a){return 0===a%4&&(0!==a%100||0===a%400)}var wb=[31,29,31,30,31,30,31,31,30,31,30,31],xb=[31,28,31,30,31,30,31,31,30,31,30,31];
function yb(a,b,c,d){function e(f,p,q){for(f="number"==typeof f?f.toString():f||"";f.length<p;)f=q[0]+f;return f}function g(f,p){return e(f,p,"0")}function k(f,p){function q(C){return 0>C?-1:0<C?1:0}var x;0===(x=q(f.getFullYear()-p.getFullYear()))&&0===(x=q(f.getMonth()-p.getMonth()))&&(x=q(f.getDate()-p.getDate()));return x}function t(f){switch(f.getDay()){case 0:return new Date(f.getFullYear()-1,11,29);case 1:return f;case 2:return new Date(f.getFullYear(),0,3);case 3:return new Date(f.getFullYear(),
0,2);case 4:return new Date(f.getFullYear(),0,1);case 5:return new Date(f.getFullYear()-1,11,31);case 6:return new Date(f.getFullYear()-1,11,30)}}function m(f){var p=f.da;for(f=new Date((new Date(f.ea+1900,0,1)).getTime());0<p;){var q=f.getMonth(),x=(vb(f.getFullYear())?wb:xb)[q];if(p>x-f.getDate())p-=x-f.getDate()+1,f.setDate(1),11>q?f.setMonth(q+1):(f.setMonth(0),f.setFullYear(f.getFullYear()+1));else{f.setDate(f.getDate()+p);break}}q=new Date(f.getFullYear()+1,0,4);p=t(new Date(f.getFullYear(),
0,4));q=t(q);return 0>=k(p,f)?0>=k(q,f)?f.getFullYear()+1:f.getFullYear():f.getFullYear()-1}var n=G[d+40>>2];d={Ma:G[d>>2],La:G[d+4>>2],ja:G[d+8>>2],oa:G[d+12>>2],ka:G[d+16>>2],ea:G[d+20>>2],X:G[d+24>>2],da:G[d+28>>2],Sa:G[d+32>>2],Ka:G[d+36>>2],Na:n?n?D(F,n):"":""};c=c?D(F,c):"";n={"%c":"%a %b %d %H:%M:%S %Y","%D":"%m/%d/%y","%F":"%Y-%m-%d","%h":"%b","%r":"%I:%M:%S %p","%R":"%H:%M","%T":"%H:%M:%S","%x":"%m/%d/%y","%X":"%H:%M:%S","%Ec":"%c","%EC":"%C","%Ex":"%m/%d/%y","%EX":"%H:%M:%S","%Ey":"%y",
"%EY":"%Y","%Od":"%d","%Oe":"%e","%OH":"%H","%OI":"%I","%Om":"%m","%OM":"%M","%OS":"%S","%Ou":"%u","%OU":"%U","%OV":"%V","%Ow":"%w","%OW":"%W","%Oy":"%y"};for(var r in n)c=c.replace(new RegExp(r,"g"),n[r]);var u="Sunday Monday Tuesday Wednesday Thursday Friday Saturday".split(" "),J="January February March April May June July August September October November December".split(" ");n={"%a":function(f){return u[f.X].substring(0,3)},"%A":function(f){return u[f.X]},"%b":function(f){return J[f.ka].substring(0,
3)},"%B":function(f){return J[f.ka]},"%C":function(f){return g((f.ea+1900)/100|0,2)},"%d":function(f){return g(f.oa,2)},"%e":function(f){return e(f.oa,2," ")},"%g":function(f){return m(f).toString().substring(2)},"%G":function(f){return m(f)},"%H":function(f){return g(f.ja,2)},"%I":function(f){f=f.ja;0==f?f=12:12<f&&(f-=12);return g(f,2)},"%j":function(f){for(var p=0,q=0;q<=f.ka-1;p+=(vb(f.ea+1900)?wb:xb)[q++]);return g(f.oa+p,3)},"%m":function(f){return g(f.ka+1,2)},"%M":function(f){return g(f.La,
2)},"%n":function(){return"\n"},"%p":function(f){return 0<=f.ja&&12>f.ja?"AM":"PM"},"%S":function(f){return g(f.Ma,2)},"%t":function(){return"\t"},"%u":function(f){return f.X||7},"%U":function(f){return g(Math.floor((f.da+7-f.X)/7),2)},"%V":function(f){var p=Math.floor((f.da+7-(f.X+6)%7)/7);2>=(f.X+371-f.da-2)%7&&p++;if(p)53==p&&(q=(f.X+371-f.da)%7,4==q||3==q&&vb(f.ea)||(p=1));else{p=52;var q=(f.X+7-f.da-1)%7;(4==q||5==q&&vb(f.ea%400-1))&&p++}return g(p,2)},"%w":function(f){return f.X},"%W":function(f){return g(Math.floor((f.da+
7-(f.X+6)%7)/7),2)},"%y":function(f){return(f.ea+1900).toString().substring(2)},"%Y":function(f){return f.ea+1900},"%z":function(f){f=f.Ka;var p=0<=f;f=Math.abs(f)/60;return(p?"+":"-")+String("0000"+(f/60*100+f%60)).slice(-4)},"%Z":function(f){return f.Na},"%%":function(){return"%"}};c=c.replace(/%%/g,"\x00\x00");for(r in n)c.includes(r)&&(c=c.replace(new RegExp(r,"g"),n[r](d)));c=c.replace(/\0\0/g,"%");r=Ja(c,!1);if(r.length>b)return 0;E.set(r,a);return r.length-1}
function zb(a,b,c,d){var e={string:n=>{var r=0;if(null!==n&&void 0!==n&&0!==n){var u=(n.length<<2)+1;r=Ab(u);na(n,F,r,u)}return r},array:n=>{var r=Ab(n.length);E.set(n,r);return r}};a=Module["_"+a];var g=[],k=0;if(d)for(var t=0;t<d.length;t++){var m=e[c[t]];m?(0===k&&(k=Bb()),g[t]=m(d[t])):g[t]=d[t]}c=a.apply(null,g);return c=function(n){0!==k&&Cb(k);return"string"===b?n?D(F,n):"":"boolean"===b?!!n:n}(c)}
function bb(a,b,c,d){a||(a=this);this.parent=a;this.Y=a.Y;this.ia=null;this.id=Wa++;this.name=b;this.mode=c;this.S={};this.T={};this.rdev=d}Object.defineProperties(bb.prototype,{read:{get:function(){return 365===(this.mode&365)},set:function(a){a?this.mode|=365:this.mode&=-366}},write:{get:function(){return 146===(this.mode&146)},set:function(a){a?this.mode|=146:this.mode&=-147}}});nb();R=Array(4096);hb(P,"/");U("/tmp",16895,0);U("/home",16895,0);U("/home/web_user",16895,0);
(()=>{U("/dev",16895,0);Ma(259,{read:()=>0,write:(b,c,d,e)=>e});ib("/dev/null",259);La(1280,Oa);La(1536,Pa);ib("/dev/tty",1280);ib("/dev/tty1",1536);var a=Ha();V("random",a);V("urandom",a);U("/dev/shm",16895,0);U("/dev/shm/tmp",16895,0)})();(()=>{U("/proc",16895,0);var a=U("/proc/self",16895,0);U("/proc/self/fd",16895,0);hb({Y:()=>{var b=Ra(a,"fd",16895,73);b.S={lookup:(c,d)=>{var e=Q[+d];if(!e)throw new O(8);c={parent:null,Y:{wa:"fake"},S:{readlink:()=>e.path}};return c.parent=c}};return b}},"/proc/self/fd")})();
var Fb={a:function(a){return Db(a+24)+24},b:function(a,b,c){(new Ca(a)).fa(b,c);Da++;throw a;},e:function(a,b,c){X=c;try{var d=Z(a);switch(b){case 0:var e=Y();return 0>e?-28:gb(d,e).fd;case 1:case 2:return 0;case 3:return d.flags;case 4:return e=Y(),d.flags|=e,0;case 5:return e=Y(),pa[e+0>>1]=2,0;case 6:case 7:return 0;case 16:case 8:return-28;case 9:return G[Eb()>>2]=28,-1;default:return-28}}catch(g){if("undefined"==typeof W||!(g instanceof O))throw g;return-g.$}},r:function(a,b,c){X=c;try{var d=
Z(a);switch(b){case 21509:case 21505:return d.tty?0:-59;case 21510:case 21511:case 21512:case 21506:case 21507:case 21508:return d.tty?0:-59;case 21519:if(!d.tty)return-59;var e=Y();return G[e>>2]=0;case 21520:return d.tty?-28:-59;case 21531:a=e=Y();if(!d.T.Fa)throw new O(59);return d.T.Fa(d,b,a);case 21523:return d.tty?0:-59;case 21524:return d.tty?0:-59;default:B("bad ioctl syscall "+b)}}catch(g){if("undefined"==typeof W||!(g instanceof O))throw g;return-g.$}},g:function(a,b,c,d){X=d;try{b=b?D(F,
b):"";var e=b;if("/"===e.charAt(0))b=e;else{if(-100===a)var g="/";else{var k=Q[a];if(!k)throw new O(8);g=k.path}if(0==e.length)throw new O(44);b=N(g+"/"+e)}var t=d?Y():0;return lb(b,c,t).fd}catch(m){if("undefined"==typeof W||!(m instanceof O))throw m;return-m.$}},p:function(){return Date.now()},o:function(){return!0},d:function(){B("")},h:rb,j:function(a,b,c){F.copyWithin(a,b,b+c)},c:function(a){var b=F.length;a>>>=0;if(2147483648<a)return!1;for(var c=1;4>=c;c*=2){var d=b*(1+.2/c);d=Math.min(d,a+
100663296);var e=Math;d=Math.max(a,d);e=e.min.call(e,2147483648,d+(65536-d%65536)%65536);a:{try{ka.grow(e-oa.byteLength+65535>>>16);qa();var g=1;break a}catch(k){}g=void 0}if(g)return!0}return!1},m:function(a,b){var c=0;tb().forEach(function(d,e){var g=b+c;e=H[a+4*e>>2]=g;for(g=0;g<d.length;++g)E[e++>>0]=d.charCodeAt(g);E[e>>0]=0;c+=d.length+1});return 0},n:function(a,b){var c=tb();H[a>>2]=c.length;var d=0;c.forEach(function(e){d+=e.length+1});H[b>>2]=d;return 0},f:function(a){try{var b=Z(a);if(null===
b.fd)throw new O(8);b.la&&(b.la=null);try{b.T.close&&b.T.close(b)}catch(c){throw c;}finally{Q[b.fd]=null}b.fd=null;return 0}catch(c){if("undefined"==typeof W||!(c instanceof O))throw c;return c.$}},q:function(a,b,c,d){try{a:{var e=Z(a);a=b;for(var g=b=0;g<c;g++){var k=H[a>>2],t=H[a+4>>2];a+=8;var m=e,n=k,r=t,u=void 0,J=E;if(0>r||0>u)throw new O(28);if(null===m.fd)throw new O(8);if(1===(m.flags&2097155))throw new O(8);if(16384===(m.node.mode&61440))throw new O(31);if(!m.T.read)throw new O(28);var f=
"undefined"!=typeof u;if(!f)u=m.position;else if(!m.seekable)throw new O(70);var p=m.T.read(m,J,n,r,u);f||(m.position+=p);var q=p;if(0>q){var x=-1;break a}b+=q;if(q<t)break}x=b}G[d>>2]=x;return 0}catch(C){if("undefined"==typeof W||!(C instanceof O))throw C;return C.$}},k:function(a,b,c,d,e){try{b=c+2097152>>>0<4194305-!!b?(b>>>0)+4294967296*c:NaN;if(isNaN(b))return 61;var g=Z(a);mb(g,b,d);Aa=[g.position>>>0,(M=g.position,1<=+Math.abs(M)?0<M?(Math.min(+Math.floor(M/4294967296),4294967295)|0)>>>0:~~+Math.ceil((M-
+(~~M>>>0))/4294967296)>>>0:0)];G[e>>2]=Aa[0];G[e+4>>2]=Aa[1];g.la&&0===b&&0===d&&(g.la=null);return 0}catch(k){if("undefined"==typeof W||!(k instanceof O))throw k;return k.$}},i:function(a,b,c,d){try{a:{var e=Z(a);a=b;for(var g=b=0;g<c;g++){var k=H[a>>2],t=H[a+4>>2];a+=8;var m=e,n=k,r=t,u=void 0,J=E;if(0>r||0>u)throw new O(28);if(null===m.fd)throw new O(8);if(0===(m.flags&2097155))throw new O(8);if(16384===(m.node.mode&61440))throw new O(31);if(!m.T.write)throw new O(28);m.seekable&&m.flags&1024&&
mb(m,0,2);var f="undefined"!=typeof u;if(!f)u=m.position;else if(!m.seekable)throw new O(70);var p=m.T.write(m,J,n,r,u,void 0);f||(m.position+=p);var q=p;if(0>q){var x=-1;break a}b+=q}x=b}H[d>>2]=x;return 0}catch(C){if("undefined"==typeof W||!(C instanceof O))throw C;return C.$}},l:function(a,b,c,d){return yb(a,b,c,d)}};
(function(){function a(e){Module.asm=e.exports;ka=Module.asm.s;qa();sa.unshift(Module.asm.t);I--;Module.monitorRunDependencies&&Module.monitorRunDependencies(I);0==I&&(null!==va&&(clearInterval(va),va=null),K&&(e=K,K=null,e()))}function b(e){a(e.instance)}function c(e){return za().then(function(g){return WebAssembly.instantiate(g,d)}).then(function(g){return g}).then(e,function(g){z("failed to asynchronously prepare wasm: "+g);B(g)})}var d={a:Fb};I++;Module.monitorRunDependencies&&Module.monitorRunDependencies(I);
if(Module.instantiateWasm)try{return Module.instantiateWasm(d,a)}catch(e){return z("Module.instantiateWasm callback failed with error: "+e),!1}(function(){return A||"function"!=typeof WebAssembly.instantiateStreaming||wa()||L.startsWith("file://")||l||"function"!=typeof fetch?c(b):fetch(L,{credentials:"same-origin"}).then(function(e){return WebAssembly.instantiateStreaming(e,d).then(b,function(g){z("wasm streaming compile failed: "+g);z("falling back to ArrayBuffer instantiation");return c(b)})})})().catch(ba);
return{}})();Module.___wasm_call_ctors=function(){return(Module.___wasm_call_ctors=Module.asm.t).apply(null,arguments)};Module._Highs_create=function(){return(Module._Highs_create=Module.asm.u).apply(null,arguments)};Module._Highs_destroy=function(){return(Module._Highs_destroy=Module.asm.v).apply(null,arguments)};Module._Highs_run=function(){return(Module._Highs_run=Module.asm.w).apply(null,arguments)};Module._Highs_passLp=function(){return(Module._Highs_passLp=Module.asm.x).apply(null,arguments)};
Module._Highs_passMip=function(){return(Module._Highs_passMip=Module.asm.y).apply(null,arguments)};Module._Highs_setBoolOptionValue=function(){return(Module._Highs_setBoolOptionValue=Module.asm.z).apply(null,arguments)};Module._Highs_setIntOptionValue=function(){return(Module._Highs_setIntOptionValue=Module.asm.A).apply(null,arguments)};Module._Highs_setDoubleOptionValue=function(){return(Module._Highs_setDoubleOptionValue=Module.asm.B).apply(null,arguments)};
Module._Highs_setStringOptionValue=function(){return(Module._Highs_setStringOptionValue=Module.asm.C).apply(null,arguments)};Module._Highs_getSolution=function(){return(Module._Highs_getSolution=Module.asm.D).apply(null,arguments)};Module._Highs_getModelStatus=function(){return(Module._Highs_getModelStatus=Module.asm.E).apply(null,arguments)};Module._Highs_changeObjectiveSense=function(){return(Module._Highs_changeObjectiveSense=Module.asm.F).apply(null,arguments)};
Module._Highs_call=function(){return(Module._Highs_call=Module.asm.G).apply(null,arguments)};Module._Highs_getNumCols=function(){return(Module._Highs_getNumCols=Module.asm.H).apply(null,arguments)};Module._Highs_getNumRows=function(){return(Module._Highs_getNumRows=Module.asm.I).apply(null,arguments)};var Eb=Module.___errno_location=function(){return(Eb=Module.___errno_location=Module.asm.J).apply(null,arguments)},Db=Module._malloc=function(){return(Db=Module._malloc=Module.asm.K).apply(null,arguments)};
Module._free=function(){return(Module._free=Module.asm.L).apply(null,arguments)};var Bb=Module.stackSave=function(){return(Bb=Module.stackSave=Module.asm.M).apply(null,arguments)},Cb=Module.stackRestore=function(){return(Cb=Module.stackRestore=Module.asm.N).apply(null,arguments)},Ab=Module.stackAlloc=function(){return(Ab=Module.stackAlloc=Module.asm.O).apply(null,arguments)};Module.___cxa_is_pointer_type=function(){return(Module.___cxa_is_pointer_type=Module.asm.P).apply(null,arguments)};
Module.cwrap=function(a,b,c,d){c=c||[];var e=c.every(g=>"number"===g);return"string"!==b&&e&&!d?Module["_"+a]:function(){return zb(a,b,c,arguments)}};var Gb;K=function Hb(){Gb||Ib();Gb||(K=Hb)};
function Ib(){function a(){if(!Gb&&(Gb=!0,Module.calledRun=!0,!la)){Module.noFSInit||ob||(ob=!0,nb(),Module.stdin=Module.stdin,Module.stdout=Module.stdout,Module.stderr=Module.stderr,Module.stdin?V("stdin",Module.stdin):jb("/dev/tty","/dev/stdin"),Module.stdout?V("stdout",null,Module.stdout):jb("/dev/tty","/dev/stdout"),Module.stderr?V("stderr",null,Module.stderr):jb("/dev/tty1","/dev/stderr"),lb("/dev/stdin",0),lb("/dev/stdout",1),lb("/dev/stderr",1));Xa=!1;Ba(sa);aa(Module);if(Module.onRuntimeInitialized)Module.onRuntimeInitialized();
if(Module.postRun)for("function"==typeof Module.postRun&&(Module.postRun=[Module.postRun]);Module.postRun.length;){var b=Module.postRun.shift();ta.unshift(b)}Ba(ta)}}if(!(0<I)){if(Module.preRun)for("function"==typeof Module.preRun&&(Module.preRun=[Module.preRun]);Module.preRun.length;)ua();Ba(ra);0<I||(Module.setStatus?(Module.setStatus("Running..."),setTimeout(function(){setTimeout(function(){Module.setStatus("")},1);a()},1)):a())}}
if(Module.preInit)for("function"==typeof Module.preInit&&(Module.preInit=[Module.preInit]);0<Module.preInit.length;)Module.preInit.pop()();Ib();window.Highs_call=Module._Highs_call;window.Highs_changeObjectiveSense=Module._Highs_changeObjectiveSense;window.Highs_create=Module._Highs_create;window.Highs_destroy=Module._Highs_destroy;window.Highs_getModelStatus=Module._Highs_getModelStatus;window.Highs_getNumCols=Module._Highs_getNumCols;window.Highs_getNumRows=Module._Highs_getNumRows;
window.Highs_getSolution=Module._Highs_getSolution;window.Highs_run=Module._Highs_run;window.Highs_setBoolOptionValue=Module.cwrap("Highs_setBoolOptionValue","number",["number","string","number"]);window.Highs_setDoubleOptionValue=Module.cwrap("Highs_setDoubleOptionValue","number",["number","string","number"]);window.Highs_setIntOptionValue=Module.cwrap("Highs_setIntOptionValue","number",["number","string","number"]);
window.Highs_setStringOptionValue=Module.cwrap("Highs_setIntOptionValue","number",["number","string","number"]);window.Highs_passLp=Module.cwrap("Highs_passLp","number",Array(7).fill("number").concat(Array(8).fill("array")));window.Highs_passMip=Module.cwrap("Highs_passMip","number",Array(7).fill("number").concat(Array(8).fill("array")));
window.Highs_getSolution=function(a,b,c){let d=Module._malloc(b+8),e=Module._malloc(b+8),g=Module._malloc(c+8),k=Module._malloc(c+8);a=Module._Highs_getSolution(a,d+8,e+8,g+8,k+8);let t=new Uint8Array(Module.HEAPU8.buffer,d+8,b);b=new Uint8Array(Module.HEAPU8.buffer,e+8,b);let m=new Uint8Array(Module.HEAPU8.buffer,g+8,c);c=new Uint8Array(Module.HEAPU8.buffer,k+8,c);Module._free(d);Module._free(e);Module._free(g);Module._free(k);return{ret:a,cv:t,cd:b,rv:m,rd:c}};


  return createHighsModule.ready
}
);
})();
export default createHighsModule;