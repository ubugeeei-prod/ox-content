// Import markdown content
import index from "../../content/index.md";
import gettingStarted from "../../content/getting-started.md";
import api from "../../content/api.md";
import examples from "../../content/examples.md";

const pages = {
  index,
  "getting-started": gettingStarted,
  api,
  examples,
};

// Simple router
function renderPage(name: string) {
  const page = pages[name as keyof typeof pages];
  if (page) {
    document.getElementById("app")!.innerHTML = page.html;
  }
}

// Initial render
renderPage("index");

// Navigation
document.addEventListener("click", (e) => {
  const target = e.target as HTMLElement;
  if (target.tagName === "A") {
    const href = target.getAttribute("href");
    if (href?.startsWith("./")) {
      e.preventDefault();
      const pageName = href.replace("./", "").replace(".md", "");
      renderPage(pageName);
    }
  }
});
