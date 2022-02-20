// TODO: Use prod module in "release" mode ?
import { createApp } from "/static/vendor/vue@3.0.9/vue.esm-browser.js";

export function createMainApp() {
  const app = createApp(App);
  app.component("Label", Label);
  app.component("SearchResults", SearchResults);
  app.component("SearchResult", SearchResult);
  return app;
}

export const App = {
  name: "App",
  template: "#App",
  data() {
    return {
      form: {
        query: "",
      },
      results: {},
    };
  },
  async mounted() {
    await this.fetchResults("");
  },
  methods: {
    async fetchResults(query) {
      // TODO: escape query
      const response = await fetch(`/api/search?q=${query}`);
      this.results = await response.json();
    },
  },
};

export const Label = {
  name: "Label",
  template: `
    <strong><slot/></strong>
  `,
};

export const SearchResults = {
  name: "SearchResults",
  template: "#SearchResults",
  props: {
    items: Array,
  },
};

export const SearchResult = {
  name: "SearchResult",
  template: "#SearchResult",
  props: {
    title: String,
    artists: Array,
    uri: String,
    collection: String,
    thumbnail_url: {
      type: String,
      required: false,
    },
  },
  computed: {
    artistsLabel() {
      return this.artists.join(", ") || "-";
    },
  },
};
