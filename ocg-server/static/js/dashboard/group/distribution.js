const platformUrls = {
  linkedin: "https://www.linkedin.com/feed/?shareActive=true",
  x: "https://x.com/compose/post",
  instagram: "https://www.instagram.com/",
};

document.addEventListener("click", async (event) => {
  const copyButton = event.target.closest("[data-copy-text]");
  if (copyButton) {
    await navigator.clipboard.writeText(copyButton.dataset.copyText);
    copyButton.textContent = "Copied";
    return;
  }

  const platformButton = event.target.closest("[data-open-platform]");
  if (platformButton) {
    const url = platformUrls[platformButton.dataset.openPlatform];
    if (url) window.open(url, "_blank", "noopener,noreferrer");
  }
});

export { platformUrls };
