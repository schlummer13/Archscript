const button = document.getElementById("theme-toggle");
const icon = document.getElementById("theme-toggle-icon");
const label = document.getElementById("theme-toggle-label");

function getPreferredTheme() {
  const savedTheme = localStorage.getItem("theme");

  if (savedTheme === "dark" || savedTheme === "light") {
    return savedTheme;
  }

  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

function applyTheme(theme) {
  const isDark = theme === "dark";

  document.documentElement.classList.toggle("dark", isDark);
  localStorage.setItem("theme", theme);

  if (icon) {
    icon.textContent = isDark ? "●" : "○";
  }

  if (label) {
    label.textContent = isDark ? "Dark" : "Light";
  }

  window.dispatchEvent(
    new CustomEvent("archscript-theme-change", {
      detail: { theme }
    })
  );
}

applyTheme(getPreferredTheme());

button?.addEventListener("click", () => {
  const nextTheme = document.documentElement.classList.contains("dark")
    ? "light"
    : "dark";

  applyTheme(nextTheme);
});