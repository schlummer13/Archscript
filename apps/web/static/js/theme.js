const button = document.getElementById("theme-toggle");

const savedTheme = localStorage.getItem("theme");

if (savedTheme === "dark") {
  document.documentElement.classList.add("dark");
}

button?.addEventListener("click", () => {
  document.documentElement.classList.toggle("dark");

  const isDark = document.documentElement.classList.contains("dark");

  localStorage.setItem("theme", isDark ? "dark" : "light");

  window.dispatchEvent(
    new CustomEvent("archscript-theme-change", {
      detail: {
        theme: isDark ? "dark" : "light"
      }
    })
  );
});