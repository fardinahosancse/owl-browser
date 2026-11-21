// Runs before the page paints, so the chosen theme never flashes the other one.
(function () {
  const pick = localStorage.getItem("owl.theme") || "system";
  const dark = pick === "dark" || (pick === "system" && matchMedia("(prefers-color-scheme: dark)").matches);
  document.documentElement.dataset.theme = dark ? "dark" : "light";
})();
