// GitHub Wide Diffs Plugin for Appify
(function () {
  const applyWideLayout = () => {
    const containers = document.querySelectorAll(
      ".container-xl, .container-lg, .diff-view, .pull-request-tab-content"
    );
    containers.forEach((el) => {
      el.style.setProperty("max-width", "98%", "important");
      el.style.setProperty("width", "98%", "important");
    });
  };

  window.addEventListener("load", applyWideLayout);
  const observer = new MutationObserver(applyWideLayout);
  observer.observe(document.body, { childList: true, subtree: true });
  applyWideLayout();
})();
