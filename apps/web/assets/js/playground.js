import { createArchScriptEditor } from "./editor.js";

let jsPdfPromise = null;

async function loadJsPDF() {
  if (!jsPdfPromise) {
    jsPdfPromise = loadExternalScript(
      "/vendor/jspdf/jspdf.umd.min.js",
      "jspdf"
    ).then((jspdfGlobal) => {
      const jsPDF = jspdfGlobal?.jsPDF;

      if (typeof jsPDF !== "function") {
        throw new Error("jsPDF loaded, but window.jspdf.jsPDF is missing.");
      }

      return jsPDF;
    });
  }

  return jsPdfPromise;
}

let pakoPromise = null;

function loadExternalScript(src, globalName) {
  return new Promise((resolve, reject) => {
    const existing = document.querySelector(`script[data-src="${src}"]`);

    if (existing) {
      if (existing.dataset.loaded === "true") {
        resolve(window[globalName]);
        return;
      }

      existing.addEventListener(
        "load",
        () => resolve(window[globalName]),
        { once: true }
      );

      existing.addEventListener("error", reject, { once: true });
      return;
    }

    const script = document.createElement("script");
    script.src = src;
    script.async = true;
    script.dataset.src = src;

    script.addEventListener(
      "load",
      () => {
        script.dataset.loaded = "true";
        resolve(window[globalName]);
      },
      { once: true }
    );

    script.addEventListener("error", reject, { once: true });

    document.head.appendChild(script);
  });
}

async function loadPako() {
  if (!pakoPromise) {
    pakoPromise = loadExternalScript("/vendor/pako/pako.min.js", "pako").then(
      (pakoGlobal) => {
        if (!pakoGlobal?.deflateRaw) {
          throw new Error("pako loaded, but deflateRaw is missing.");
        }

        return pakoGlobal;
      }
    );
  }

  return pakoPromise;
}

let wasmModulePromise = null;
let compileArchScript = null;

async function loadWasm() {
  if (!wasmModulePromise) {
    wasmModulePromise = Function(
      "url",
      "return import(url)"
    )("/wasm/archscript_wasm.js").then(async (mod) => {
      if (typeof mod.default === "function") {
        await mod.default({
          module_or_path: "/wasm/archscript_wasm_bg.wasm"
        });
      }

      compileArchScript = mod.compile_archscript;

      if (typeof compileArchScript !== "function") {
        throw new Error("compile_archscript was not exported by the WASM module.");
      }

      return mod;
    });
  }

  return wasmModulePromise;
}

const DEFAULT_SOURCE = `project Demo

domain CustomerManagement {
  element frontend "Web App" as webapp {
    owner "Frontend Team"
    monitoring "Grafana"
    tags [production]
  }

  element service "Customer API" as api {
    owner "Platform Team"
    monitoring "Grafana"
    tags [critical, production]
  }

  element database PostgreSQL as db {
    owner "Database Team"
    monitoring "Grafana"
  }
}

relation webapp -> api {
  protocol "REST"
  tags [sync, public]
}

relation api -> db {
  protocol "PostgreSQL"
}`;

const TEMPLATES = {
  default: DEFAULT_SOURCE,

  blank: `project NewArchitecture

element frontend "Web App" as webapp
element service "API" as api
element database "Database" as db

relation webapp -> api : HTTPS
relation api -> db : SQL`,

  webapp: `project WebApplication

boundary Client {
  element actor "User" as user
  element frontend "Web App" as webapp {
    tech "React"
    owner "Frontend Team"
    monitoring "Grafana"
    tags [production]
  }
}

boundary Backend {
  element service "API" as api {
    tech "Laravel"
    owner "Backend Team"
    monitoring "Grafana"
    tags [critical, production]
  }

  element database "PostgreSQL" as db {
    owner "Database Team"
    monitoring "Grafana"
    tags [production]
  }
}

relation user -> webapp : uses
relation webapp -> api {
  protocol "REST"
  tags [sync, public]
}
relation api -> db {
  protocol "PostgreSQL"
  tags [sync]
}`,

  microservices: `project MicroservicesPlatform

domain Identity {
  element service "Auth Service" as auth {
    tech "Rust"
    owner "Identity Team"
    monitoring "Grafana"
    tags [critical, production]
  }

  element database "Auth Database" as auth_db {
    owner "Identity Team"
    monitoring "Grafana"
    tags [production]
  }
}

domain Billing {
  element service "Billing Service" as billing {
    tech "Go"
    owner "Billing Team"
    monitoring "Grafana"
    tags [critical, production]
  }

  element database "Billing Database" as billing_db {
    owner "Billing Team"
    monitoring "Grafana"
    tags [production]
  }
}

boundary Messaging {
  element event_bus "Event Bus" as event_bus {
    tech "Kafka"
    owner "Platform Team"
    monitoring "Grafana"
    tags [critical, production]
  }
}

relation auth -> auth_db : PostgreSQL
relation billing -> billing_db : PostgreSQL
relation auth -> event_bus {
  protocol "Kafka"
  label "publishes UserCreated"
}
relation billing -> event_bus {
  protocol "Kafka"
  label "subscribes UserCreated"
}`,

  ddd: `project DDDArchitecture

domain CustomerManagement {
  element service "Customer Service" as customer_service {
    owner "Customer Team"
    monitoring "Grafana"
    tags [critical, production]
  }

  element database "Customer Database" as customer_db {
    owner "Customer Team"
    monitoring "Grafana"
    tags [production]
  }
}

domain Ordering {
  element service "Order Service" as order_service {
    owner "Order Team"
    monitoring "Grafana"
    tags [critical, production]
  }

  element database "Order Database" as order_db {
    owner "Order Team"
    monitoring "Grafana"
    tags [production]
  }
}

domain Billing {
  element service "Billing Service" as billing_service {
    owner "Billing Team"
    monitoring "Grafana"
    tags [critical, production]
  }
}

relation customer_service -> customer_db : owns
relation order_service -> order_db : owns
relation order_service -> customer_service : reads customer profile
relation order_service -> billing_service : requests payment`,

  event: `project EventDrivenArchitecture

element frontend "Admin UI" as admin_ui {
  owner "Frontend Team"
  monitoring "Grafana"
  tags [production]
}

element service "Command API" as command_api {
  owner "Platform Team"
  monitoring "Grafana"
  tags [critical, production]
}

element event_bus "Event Bus" as event_bus {
  tech "Kafka"
  owner "Platform Team"
  monitoring "Grafana"
  tags [critical, production]
}

element service "Projection Worker" as projection_worker {
  owner "Platform Team"
  monitoring "Grafana"
  tags [production]
}

element database "Read Model" as read_model {
  owner "Platform Team"
  monitoring "Grafana"
  tags [production]
}

relation admin_ui -> command_api : REST
relation command_api -> event_bus {
  label "publishes Commands"
  protocol "Kafka"
}
relation projection_worker -> event_bus {
  label "subscribes Events"
  protocol "Kafka"
}
relation projection_worker -> read_model : writes`,

  saas: `project SaaSPlatform

environment Production {
  element cdn "CDN" as cdn {
    tech "Cloudflare"
    owner "Platform Team"
    monitoring "Cloudflare"
    tags [production]
  }

  element frontend "Customer Portal" as portal {
    tech "React"
    owner "Frontend Team"
    monitoring "Grafana"
    tags [production]
  }

  element gateway "API Gateway" as gateway {
    owner "Platform Team"
    monitoring "Grafana"
    tags [critical, production]
  }

  element service "Tenant API" as tenant_api {
    owner "Backend Team"
    monitoring "Grafana"
    tags [critical, production]
  }

  element database "Tenant Database" as tenant_db {
    owner "Database Team"
    monitoring "Grafana"
    tags [critical, production]
  }

  element identity_provider "Identity Provider" as idp {
    tech "Keycloak"
    owner "Security Team"
    monitoring "Grafana"
    tags [critical, production]
  }
}

relation cdn -> portal : serves
relation portal -> gateway : HTTPS
relation gateway -> tenant_api : REST
relation tenant_api -> tenant_db : PostgreSQL
relation portal -> idp : OIDC`
};

const sourceContainer = document.getElementById("source");
const renderBtn = document.getElementById("render-btn");
const plantumlEl = document.getElementById("plantuml");
const documentationEl = document.getElementById("documentation");
const diagramEl = document.getElementById("diagram");
const violationsEl = document.getElementById("violations");
const diagnosticsEl = document.getElementById("diagnostics");
const downloadSourceBtn = document.getElementById("download-source-btn");
const downloadPumlBtn = document.getElementById("download-puml-btn");
const templateSelect = document.getElementById("template-select");
const downloadDocsBtn = document.getElementById("download-docs-btn");
const downloadSvgBtn = document.getElementById("download-svg-btn");
const downloadPngBtn = document.getElementById("download-png-btn");
const downloadPdfBtn = document.getElementById("download-pdf-btn");

let editor = null;
let latestSvg = null;
let latestResult = null;
let debounceTimer = null;
let currentRenderId = 0;

const PLANTUML_ALPHABET =
  "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz-_";

function encode6Bit(value) {
  if (value < 0) return "";
  if (value < 64) return PLANTUML_ALPHABET[value];
  return "";
}

function append3Bytes(byte1, byte2, byte3) {
  const c1 = byte1 >> 2;
  const c2 = ((byte1 & 0x3) << 4) | (byte2 >> 4);
  const c3 = ((byte2 & 0xf) << 2) | (byte3 >> 6);
  const c4 = byte3 & 0x3f;

  return (
    encode6Bit(c1 & 0x3f) +
    encode6Bit(c2 & 0x3f) +
    encode6Bit(c3 & 0x3f) +
    encode6Bit(c4 & 0x3f)
  );
}

async function encodePlantUml(source) {
  const pako = await loadPako();

  const utf8 = new TextEncoder().encode(source);
  const deflated = pako.deflateRaw(utf8, { level: 9 });

  let encoded = "";

  for (let i = 0; i < deflated.length; i += 3) {
    if (i + 2 === deflated.length) {
      encoded += append3Bytes(deflated[i], deflated[i + 1], 0);
    } else if (i + 1 === deflated.length) {
      encoded += append3Bytes(deflated[i], 0, 0);
    } else {
      encoded += append3Bytes(deflated[i], deflated[i + 1], deflated[i + 2]);
    }
  }

  return encoded;
}

function clearElement(element) {
  if (element) {
    element.innerHTML = "";
  }
}

function setText(element, text) {
  if (element) {
    element.textContent = text;
  }
}

async function renderDiagram(plantuml) {
  const renderId = ++currentRenderId;

  clearElement(diagramEl);

  if (!plantuml) {
    setText(diagramEl, "No diagram generated.");
    latestSvg = null;
    return;
  }

  setText(diagramEl, "Rendering diagram...");

  const encoded = await encodePlantUml(plantuml);
  const url = `https://www.plantuml.com/plantuml/svg/${encoded}`;

  try {
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(`PlantUML request failed: ${response.status}`);
    }

    const svg = await response.text();

    if (renderId !== currentRenderId) {
      return;
    }

    latestSvg = svg;

    if (diagramEl) {
      diagramEl.textContent = "";
      diagramEl.innerHTML = svg;
    }
  } catch (error) {
    console.error(error);

    if (renderId !== currentRenderId) {
      return;
    }

    latestSvg = null;
    setText(diagramEl, "Could not render PlantUML diagram.");
  }
}

function renderNotices(target, items, type) {
  if (!target) {
    return;
  }

  target.innerHTML = "";

  if (!items || items.length === 0) {
    return;
  }

  for (const item of items) {
    const div = document.createElement("div");

    div.className =
      type === "error"
        ? "mb-3 border border-[var(--border)] border-l-4 border-l-[var(--error)] bg-[var(--surface)] p-4 text-sm leading-6"
        : "mb-3 border border-[var(--border)] border-l-4 border-l-[var(--warning)] bg-[var(--surface)] p-4 text-sm leading-6";

    const title = document.createElement("strong");
    title.className = "mb-1 block text-[11px] uppercase tracking-[0.14em]";
    title.textContent = item.rule || "Diagnostic";

    const message = document.createElement("span");
    message.textContent = item.message || "";

    div.appendChild(title);
    div.appendChild(message);
    target.appendChild(div);
  }
}

async function compileAndRender() {
  if (!editor) {
    return;
  }

  try {
    localStorage.setItem("archscript-source", editor.getValue());

    await loadWasm();

    if (typeof compileArchScript !== "function") {
      throw new Error("ArchScript WASM compiler is not available.");
    }

    const raw = compileArchScript(editor.getValue());
    const result = JSON.parse(raw);

    latestResult = result;

    setText(plantumlEl, result.plantuml || "");
    setText(documentationEl, result.documentation || "");

    renderNotices(violationsEl, result.violations, "warning");
    renderNotices(diagnosticsEl, result.diagnostics, "error");

    await renderDiagram(result.plantuml);
  } catch (error) {
    console.error(error);

    latestResult = null;
    latestSvg = null;

    setText(diagramEl, "Could not compile ArchScript.");
    setText(plantumlEl, "");
    setText(documentationEl, "");

    if (diagnosticsEl) {
      diagnosticsEl.innerHTML = "";

      const div = document.createElement("div");
      div.className =
        "mb-3 border border-[var(--border)] border-l-4 border-l-[var(--error)] bg-[var(--surface)] p-4 text-sm leading-6";

      const title = document.createElement("strong");
      title.className = "mb-1 block text-[11px] uppercase tracking-[0.14em]";
      title.textContent = "Runtime Error";

      const message = document.createElement("span");
      message.textContent = error.message || "Unknown error";

      div.appendChild(title);
      div.appendChild(message);
      diagnosticsEl.appendChild(div);
    }
  }
}

function scheduleCompile() {
  clearTimeout(debounceTimer);

  debounceTimer = setTimeout(() => {
    compileAndRender();
  }, 300);
}

function downloadFile(filename, content, mimeType = "text/plain") {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);

  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  link.click();

  URL.revokeObjectURL(url);
}

function initEditor() {
  if (!sourceContainer) {
    throw new Error("Editor container #source was not found.");
  }

  editor = createArchScriptEditor({
    parent: sourceContainer,
    value: localStorage.getItem("archscript-source") || DEFAULT_SOURCE,
    onChange: () => {
      scheduleCompile();
    }
  });
}

function initEvents() {
  templateSelect?.addEventListener("change", () => {
    const template = TEMPLATES[templateSelect.value];

    if (!template || !editor) {
      return;
    }

    editor.setValue(template);
    localStorage.setItem("archscript-source", template);
    compileAndRender();
  });

  renderBtn?.addEventListener("click", compileAndRender);

  downloadSourceBtn?.addEventListener("click", () => {
    if (!editor) return;

    downloadFile("architecture.archscript", editor.getValue());
  });

  downloadPumlBtn?.addEventListener("click", () => {
    if (!latestResult?.plantuml) return;

    downloadFile("architecture.puml", latestResult.plantuml);
  });

  downloadDocsBtn?.addEventListener("click", () => {
    if (!latestResult?.documentation) return;

    downloadFile(
      "architecture.md",
      latestResult.documentation,
      "text/markdown"
    );
  });

  downloadSvgBtn?.addEventListener("click", () => {
    if (!latestSvg) return;

    downloadFile(
      "architecture.svg",
      latestSvg,
      "image/svg+xml"
    );
  });

  downloadPngBtn?.addEventListener("click", async () => {
    if (!latestSvg) return;

    const svgBlob = new Blob([latestSvg], {
      type: "image/svg+xml"
    });

    const svgUrl = URL.createObjectURL(svgBlob);
    const image = new Image();

    image.onload = () => {
      const canvas = document.createElement("canvas");

      canvas.width = image.width * 2;
      canvas.height = image.height * 2;

      const context = canvas.getContext("2d");

      context.fillStyle = "#ffffff";
      context.fillRect(0, 0, canvas.width, canvas.height);

      context.drawImage(
        image,
        0,
        0,
        canvas.width,
        canvas.height
      );

      canvas.toBlob((blob) => {
        if (!blob) return;

        const pngUrl = URL.createObjectURL(blob);
        const link = document.createElement("a");

        link.href = pngUrl;
        link.download = "architecture.png";
        link.click();

        URL.revokeObjectURL(pngUrl);
        URL.revokeObjectURL(svgUrl);
      }, "image/png");
    };

    image.onerror = () => {
      URL.revokeObjectURL(svgUrl);
      setText(diagramEl, "Could not export PNG.");
    };

    image.src = svgUrl;
  });
}

function initPlayground() {
  try {
    initEditor();
    initEvents();

    compileAndRender();
  } catch (error) {
    console.error(error);

    setText(diagramEl, "Could not initialize playground.");

    if (diagnosticsEl) {
      diagnosticsEl.textContent = error.message || "Unknown initialization error";
    }
  }
}

async function exportArchitecturePdf({ result, svg }) {
  const jsPDF = await loadJsPDF();

  const pdf = new jsPDF({
    orientation: "portrait",
    unit: "mm",
    format: "a4",
    putOnlyUsedFonts: true,
    compress: true
  });

  const meta = extractProjectMeta(result);

  drawCoverPage(pdf, meta, result);
  await drawDiagramPage(pdf, meta, svg);
  drawDocumentationPages(pdf, meta, result.documentation || "");
  drawReviewPage(pdf, meta, result);

  pdf.save(`${slugify(meta.projectName)}-architecture-report.pdf`);
}

function extractProjectMeta(result) {
  const projectName =
    result?.project?.name ||
    "Architecture";

  return {
    projectName,
    generatedAt: new Date(),
    product: "ArchScript",
    website: "archscript.dev",
    version: "SYSTEM / V0.9"
  };
}

function drawPageFrame(pdf, meta, title) {
  const pageWidth = pdf.internal.pageSize.getWidth();
  const pageHeight = pdf.internal.pageSize.getHeight();

  pdf.setDrawColor(17, 17, 17);
  pdf.setLineWidth(0.25);

  pdf.line(12, 14, pageWidth - 12, 14);
  pdf.line(12, pageHeight - 16, pageWidth - 12, pageHeight - 16);

  pdf.setFont("helvetica", "bold");
  pdf.setFontSize(9);
  pdf.text("ARCHSCRIPT", 12, 10);

  pdf.setFont("helvetica", "normal");
  pdf.setFontSize(7);
  pdf.text("ARCHITECTURE AS CODE", 12, 13);

  pdf.setFont("helvetica", "normal");
  pdf.setFontSize(8);
  pdf.text(meta.version, pageWidth - 12, 10, {
    align: "right"
  });

  pdf.setFont("helvetica", "bold");
  pdf.setFontSize(10);
  pdf.text(title.toUpperCase(), 12, 23);

  pdf.setFont("helvetica", "normal");
  pdf.setFontSize(8);
  pdf.text(
    `Generated by ${meta.website}`,
    12,
    pageHeight - 10
  );

  pdf.text(
    formatDate(meta.generatedAt),
    pageWidth - 12,
    pageHeight - 10,
    {
      align: "right"
    }
  );
}

function drawCoverPage(pdf, meta, result) {
  const pageWidth = pdf.internal.pageSize.getWidth();

  drawPageFrame(pdf, meta, "Architecture Report");

  pdf.setFont("helvetica", "bold");
  pdf.setFontSize(34);
  pdf.text(meta.projectName, 20, 62, {
    maxWidth: pageWidth - 40
  });

  pdf.setFontSize(12);
  pdf.setFont("helvetica", "normal");
  pdf.text(
    "Software architecture report generated from ArchScript DSL.",
    20,
    78,
    {
      maxWidth: pageWidth - 40
    }
  );

  const stats = collectStats(result);

  drawMetricBox(pdf, 20, 105, "ELEMENTS", String(stats.elements));
  drawMetricBox(pdf, 72, 105, "RELATIONS", String(stats.relations));
  drawMetricBox(pdf, 124, 105, "VIOLATIONS", String(stats.violations));

  pdf.setFont("helvetica", "bold");
  pdf.setFontSize(10);
  pdf.text("REPORT CONTENT", 20, 150);

  pdf.setFont("helvetica", "normal");
  pdf.setFontSize(10);

  const items = [
    "01 / System diagram",
    "02 / Architecture documentation",
    "03 / Rule engine review",
    "04 / Diagnostics"
  ];

  let y = 162;

  for (const item of items) {
    pdf.text(item, 20, y);
    y += 9;
  }
}

function drawMetricBox(pdf, x, y, label, value) {
  pdf.setDrawColor(17, 17, 17);
  pdf.setLineWidth(0.25);
  pdf.rect(x, y, 42, 28);

  pdf.setFont("helvetica", "normal");
  pdf.setFontSize(7);
  pdf.text(label, x + 4, y + 8);

  pdf.setFont("helvetica", "bold");
  pdf.setFontSize(18);
  pdf.text(value, x + 4, y + 22);
}

async function drawDiagramPage(pdf, meta, svg) {
  pdf.addPage();

  drawPageFrame(pdf, meta, "System View");

  const imageData = await svgToPngDataUrl(svg, 2);
  const size = await getImageSize(imageData);

  const pageWidth = pdf.internal.pageSize.getWidth();
  const pageHeight = pdf.internal.pageSize.getHeight();

  const marginX = 16;
  const top = 32;
  const bottom = 24;

  const maxWidth = pageWidth - marginX * 2;
  const maxHeight = pageHeight - top - bottom;

  const ratio = size.width / size.height;

  let renderWidth = maxWidth;
  let renderHeight = renderWidth / ratio;

  if (renderHeight > maxHeight) {
    renderHeight = maxHeight;
    renderWidth = renderHeight * ratio;
  }

  const x = (pageWidth - renderWidth) / 2;

  pdf.addImage(
    imageData,
    "PNG",
    x,
    top,
    renderWidth,
    renderHeight
  );
}

function drawDocumentationPages(pdf, meta, markdown) {
  if (!markdown.trim()) {
    return;
  }

  const sections = parseMarkdownForPdf(markdown);

  pdf.addPage();
  drawPageFrame(pdf, meta, "Architecture Documentation");

  let y = 34;

  for (const block of sections) {
    const pageHeight = pdf.internal.pageSize.getHeight();

    if (y > pageHeight - 32) {
      pdf.addPage();
      drawPageFrame(pdf, meta, "Architecture Documentation");
      y = 34;
    }

    if (block.type === "h1") {
      pdf.setFont("helvetica", "bold");
      pdf.setFontSize(18);
      pdf.text(block.text, 16, y);
      y += 12;
      continue;
    }

    if (block.type === "h2") {
      y += 4;
      pdf.setFont("helvetica", "bold");
      pdf.setFontSize(13);
      pdf.text(block.text, 16, y);
      y += 9;
      continue;
    }

    if (block.type === "h3") {
      y += 2;
      pdf.setFont("helvetica", "bold");
      pdf.setFontSize(10);
      pdf.text(block.text, 16, y);
      y += 7;
      continue;
    }

    if (block.type === "bullet") {
      pdf.setFont("helvetica", "normal");
      pdf.setFontSize(9);

      const lines = pdf.splitTextToSize(
        `• ${block.text}`,
        174
      );

      for (const line of lines) {
        if (y > pageHeight - 26) {
          pdf.addPage();
          drawPageFrame(pdf, meta, "Architecture Documentation");
          y = 34;
        }

        pdf.text(line, 20, y);
        y += 5.5;
      }

      continue;
    }

    pdf.setFont("helvetica", "normal");
    pdf.setFontSize(9);

    const lines = pdf.splitTextToSize(block.text, 178);

    for (const line of lines) {
      if (y > pageHeight - 26) {
        pdf.addPage();
        drawPageFrame(pdf, meta, "Architecture Documentation");
        y = 34;
      }

      pdf.text(line, 16, y);
      y += 5.5;
    }
  }
}

function drawReviewPage(pdf, meta, result) {
  pdf.addPage();

  drawPageFrame(pdf, meta, "Architecture Review");

  let y = 36;

  pdf.setFont("helvetica", "bold");
  pdf.setFontSize(13);
  pdf.text("Rule Engine", 16, y);
  y += 10;

  const violations = result.violations || [];

  if (violations.length === 0) {
    drawStatusBox(
      pdf,
      16,
      y,
      "NO RULE VIOLATIONS",
      "The architecture passed all currently enabled ArchScript rules."
    );

    y += 34;
  } else {
    for (const violation of violations) {
      if (y > 250) {
        pdf.addPage();
        drawPageFrame(pdf, meta, "Architecture Review");
        y = 36;
      }

      drawStatusBox(
        pdf,
        16,
        y,
        violation.rule,
        violation.message
      );

      y += 34;
    }
  }

  y += 8;

  pdf.setFont("helvetica", "bold");
  pdf.setFontSize(13);
  pdf.text("Diagnostics", 16, y);
  y += 10;

  const diagnostics = result.diagnostics || [];

  if (diagnostics.length === 0) {
    drawStatusBox(
      pdf,
      16,
      y,
      "NO DIAGNOSTICS",
      "No parser or validation diagnostics were reported."
    );

    return;
  }

  for (const diagnostic of diagnostics) {
    if (y > 250) {
      pdf.addPage();
      drawPageFrame(pdf, meta, "Architecture Review");
      y = 36;
    }

    drawStatusBox(
      pdf,
      16,
      y,
      "DIAGNOSTIC",
      diagnostic.message
    );

    y += 34;
  }
}

function drawStatusBox(pdf, x, y, title, text) {
  pdf.setDrawColor(17, 17, 17);
  pdf.setLineWidth(0.25);
  pdf.rect(x, y, 178, 24);

  pdf.setFont("helvetica", "bold");
  pdf.setFontSize(8);
  pdf.text(title.toUpperCase(), x + 4, y + 7);

  pdf.setFont("helvetica", "normal");
  pdf.setFontSize(8);

  const lines = pdf.splitTextToSize(text, 166);
  pdf.text(lines, x + 4, y + 14);
}

function parseMarkdownForPdf(markdown) {
  return markdown
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      if (line.startsWith("# ")) {
        return {
          type: "h1",
          text: line.replace(/^# /, "")
        };
      }

      if (line.startsWith("## ")) {
        return {
          type: "h2",
          text: line.replace(/^## /, "")
        };
      }

      if (line.startsWith("### ")) {
        return {
          type: "h3",
          text: line.replace(/^### /, "")
        };
      }

      if (line.startsWith("- ")) {
        return {
          type: "bullet",
          text: line.replace(/^- /, "")
        };
      }

      return {
        type: "paragraph",
        text: line
      };
    });
}

async function svgToPngDataUrl(svg, scale = 2) {
  const svgBlob = new Blob([svg], {
    type: "image/svg+xml;charset=utf-8"
  });

  const svgUrl = URL.createObjectURL(svgBlob);
  const image = new Image();

  const loaded = new Promise((resolve, reject) => {
    image.onload = () => resolve();
    image.onerror = reject;
  });

  image.src = svgUrl;

  await loaded;

  const canvas = document.createElement("canvas");

  canvas.width = image.width * scale;
  canvas.height = image.height * scale;

  const context = canvas.getContext("2d");

  context.fillStyle = "#ffffff";
  context.fillRect(0, 0, canvas.width, canvas.height);

  context.drawImage(
    image,
    0,
    0,
    canvas.width,
    canvas.height
  );

  URL.revokeObjectURL(svgUrl);

  return canvas.toDataURL("image/png");
}

function getImageSize(dataUrl) {
  return new Promise((resolve, reject) => {
    const image = new Image();

    image.onload = () => {
      resolve({
        width: image.width,
        height: image.height
      });
    };

    image.onerror = reject;
    image.src = dataUrl;
  });
}

function collectStats(result) {
  const project = result.project;

  if (!project) {
    return {
      elements: 0,
      relations: 0,
      violations: 0
    };
  }

  let elements = project.elements?.length || 0;

  for (const boundary of project.boundaries || []) {
    elements += boundary.elements?.length || 0;
  }

  for (const domain of project.domains || []) {
    elements += domain.elements?.length || 0;
  }

  for (const environment of project.environments || []) {
    elements += environment.elements?.length || 0;
  }

  return {
    elements,
    relations: project.relations?.length || 0,
    violations: result.violations?.length || 0
  };
}

function formatDate(date) {
  return new Intl.DateTimeFormat("de-DE", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit"
  }).format(date);
}

function slugify(value) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/(^-|-$)/g, "");
}

downloadPdfBtn?.addEventListener("click", async () => {
  if (!latestResult || !latestSvg) {
    return;
  }

  const originalText = downloadPdfBtn.textContent;

  downloadPdfBtn.textContent = "Exporting...";
  downloadPdfBtn.disabled = true;

  try {
    await exportArchitecturePdf({
      result: latestResult,
      svg: latestSvg
    });
  } catch (error) {
    console.error(error);
    alert(error.message || "Could not export PDF.");
  } finally {
    downloadPdfBtn.textContent = originalText;
    downloadPdfBtn.disabled = false;
  }
});

initPlayground();