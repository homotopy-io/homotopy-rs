
const highs_settings = {};
createHighsModule(highs_settings).then(highs => {
	window.highs = highs;
	window.Highs_call = highs._Highs_call;
	window.Highs_changeObjectiveSense = highs._Highs_changeObjectiveSense;
	window.Highs_create = highs._Highs_create;
	window.Highs_destroy = highs._Highs_destroy;
	window.Highs_getModelStatus = highs._Highs_getModelStatus;
	window.Highs_getNumCols = highs._Highs_getNumCols;
	window.Highs_getNumRows = highs._Highs_getNumRows;
	window.Highs_getSolution = highs._Highs_getSolution;
	window.Highs_passLp = highs.cwrap("Highs_passLp","number",["number","number","number","number","number","number","number","array","array","array","array","array","array","array","array","array"]);
	window.Highs_passMip = highs.cwrap("Highs_passMip","number",["number","number","number","number","number","number","number","array","array","array","array","array","array","array","array","array"]);
	window.Highs_run = highs._Highs_run;
	window.Highs_setBoolOptionValue = highs.cwrap("Highs_setBoolOptionValue","number",["number", "string", "number"]);
	window.Highs_setDoubleOptionValue = highs.cwrap("Highs_setDoubleOptionValue","number",["number", "string", "number"]);
	window.Highs_setIntOptionValue = highs.cwrap("Highs_setIntOptionValue","number",["number", "string", "number"]);
	window.Highs_setStringOptionValue = highs.cwrap("Highs_setIntOptionValue","number",["number", "string", "number"]);
	window.Highs_getSolution = function(h,c,r) {
		let ptr0=highs._malloc(c+8);
		let ptr1=highs._malloc(c+8);
		let ptr2=highs._malloc(r+8);
		let ptr3=highs._malloc(r+8);
		let ret=highs._Highs_getSolution(h,ptr0+8,ptr1+8,ptr2+8,ptr3+8);
		let cv=new Uint8Array(highs.HEAPU8.buffer,ptr0+8,c);
		let cd=new Uint8Array(highs.HEAPU8.buffer,ptr1+8,c);
		let rv=new Uint8Array(highs.HEAPU8.buffer,ptr2+8,r);
		let rd=new Uint8Array(highs.HEAPU8.buffer,ptr3+8,r);
		highs._free(ptr0);
		highs._free(ptr1);
		highs._free(ptr2);
		highs._free(ptr3);
		return {"ret": ret, "cv": cv, "cd": cd, "rv": rv, "rd": rd};
	};
}).catch(console.err);
