// uxui theme mode: light, dark, or the system's (no data-uxui-mode). Load in <head> so the page never flashes.
(function () {
  var key = "uxui-theme";
  var root = document.documentElement;
  var stored = null;
  try { stored = localStorage.getItem(key); } catch (e) {}
  if (stored === "light" || stored === "dark") root.setAttribute("data-uxui-mode", stored);

  function current() {
    var set = root.getAttribute("data-uxui-mode");
    if (set) return set;
    return window.matchMedia && matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }

  function label(button) {
    var next = current() === "dark" ? "light" : "dark";
    button.textContent = next === "dark" ? "☾ Dark" : "☀ Light";
    button.setAttribute("aria-label", "Switch to " + next + " mode");
  }

  document.addEventListener("DOMContentLoaded", function () {
    var button = document.createElement("button");
    button.type = "button";
    button.className = "uxui-theme-toggle";
    label(button);
    button.addEventListener("click", function () {
      var next = current() === "dark" ? "light" : "dark";
      root.setAttribute("data-uxui-mode", next);
      try { localStorage.setItem(key, next); } catch (e) {}
      label(button);
    });
    document.body.appendChild(button);
  });
})();
