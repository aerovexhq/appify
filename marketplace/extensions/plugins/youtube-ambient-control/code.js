// YouTube Ambient Enhancer Plugin for Appify
(function () {
  const optimizePlayer = () => {
    const ambient = document.getElementById("cinematic-container");
    if (ambient) {
      ambient.style.setProperty("display", "none", "important");
    }
  };

  window.addEventListener("load", optimizePlayer);
  setInterval(optimizePlayer, 2000);
})();
