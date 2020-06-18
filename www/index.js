import * as lalr1 from "lalr1-frontend";
import * as svgPanZoom from "svg-pan-zoom";
import * as ace from "ace-builds";
import Split from 'split.js'

const resizeEvent = new Event("paneresize");
Split(["#editor", "#graph"], {
  sizes: [25, 75],
  onDragEnd: function () {
    const svgOutput = document.getElementById("svg_output");
    if (svgOutput != null) {
      svgOutput.dispatchEvent(resizeEvent);
    }
  }
});

const editor = ace.edit("editor");
editor.getSession().setMode("ace/mode/dot");

const parser = new DOMParser();
let result;

const output = document.querySelector("#output");
const error = document.querySelector("#error");
const algorithm = document.querySelector("#algorithm select");
const format = document.querySelector("#format select");
const raw = document.querySelector("#raw input");

function updateGraph() {
  output.classList.add("working");
  output.classList.remove("error");
  const text = editor.getSession().getDocument().getValue();
  const algo = algorithm.value;
  try {
    result = algo === "dfa" ? lalr1.lexer(text) : lalr1.parser(text, algo, raw.checked);
  } catch (msg) {
    output.classList.add("error");
    while (error.firstChild) {
      error.removeChild(error.firstChild);
    }
    error.appendChild(document.createTextNode(msg));
  }
  output.classList.remove("working");
  updateOutput();
}

function updateOutput() {
  const output = document.querySelector("#output");
  let svg = output.querySelector("svg");
  if (svg) output.removeChild(svg);
  let text = output.querySelector("#text");
  if (text) output.removeChild(text);
  let img = output.querySelector("img");
  if (img) output.removeChild(img);

  if (!result) return;
  if (raw.checked || algorithm.value === "ll(1)") { // render text
    const text = document.createElement("div");
    text.id = "text";
    text.appendChild(document.createTextNode(result));
    output.appendChild(text);
  } else if (format.value === "svg") { // render svg
    const svg = parser.parseFromString(Viz(result), "image/svg+xml").documentElement;
    svg.id = "svg_output";
    output.appendChild(svg);
    const panZoom = svgPanZoom(svg, {
      zoomEnabled: true,
      controlIconsEnabled: true,
      fit: true,
      center: true,
      minZoom: 0.1
    });
    svg.addEventListener("paneresize", _ => panZoom.resize(), false);
    window.addEventListener("resize", _ => panZoom.resize());
  } else { // render png
    output.appendChild(Viz.svgXmlToPngImageElement(Viz(result)));
  }
}

editor.on("change", () => updateGraph());
algorithm.addEventListener("change", () => updateGraph());
format.addEventListener("change", () => updateOutput());
raw.addEventListener("change", () => updateGraph());

updateGraph();