// TODO: Use prod module in "release" mode ?
import { createApp } from "/static/vendor/vue@3.2.31/vue.esm-browser.js";

import {
  App,
  Label,
  SearchResult,
  SearchResults,
  SearchResultSkeleton,
  Skeleton,
} from "./components/index.js";
import { installLazyImage } from "./components/LazyImage.js";

export function createMainApp() {
  const app = createApp(App);
  app.component("Label", Label);
  app.component("SearchResults", SearchResults);
  app.component("SearchResult", SearchResult);
  app.component("SearchResultSkeleton", SearchResultSkeleton);
  app.component("Skeleton", Skeleton);
  installLazyImage(app);
  return app;
}
