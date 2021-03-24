import * as wasm from "epubparse";

wasm.greet();

const inputElement = document.getElementById("input");
inputElement.addEventListener("change", handleFiles, false);
function handleFiles() {
    if (inputElement.files.length == 0) {
        return
    }
    let file = inputElement.files[0]
    console.log(file.name)
    const filereader = new FileReader()
    filereader.onload = function (e) {
        let data = new Uint8Array(filereader.result)
        let title = wasm.parse_epub(data)
        alert(title)
    }
    console.log("about to read in file")
    filereader.readAsArrayBuffer(file)
}