// TODO: Use prod module in "release" mode ?
import {
  createApp,
  onMounted,
  reactive,
  ref,
} from "/static/vendor/vue@3.2.31/vue.esm-browser.js";

export function createMainApp() {
  const app = createApp(App);
  app.component("Label", Label);
  app.component("SearchResults", SearchResults);
  app.component("SearchResult", SearchResult);
  return app;
}

export const App = {
  name: "App",
  template: `
    <h1>Spotify Search</h1>
    <div>
      <form @submit.prevent="fetchResults">
        <input v-model="formData.query" placeholder="Search terms..." class="search-input">
      </form>
    </div>
    <SearchResults :items="results.items"/>
  `,
  setup() {
    const formData = reactive({ query: "" });
    const results = ref([]);

    const fetchResults = async () => {
      const query = formData.query;
      // TODO: escape query
      const response = await fetch(`/api/search?q=${query}`);
      results.value = await response.json();
    };

    onMounted(() => {
      fetchResults();
    });

    return { fetchResults, formData, results };
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
  template: `
    <div class="container mrgt+">
      <Label>Search results</Label>
      <div class="mrgt+">
        <SearchResult v-if="items" v-for="item in items" v-bind="item" class="row"/>
      </div>
    </div>
  `,
  props: {
    items: Array,
  },
};

export const SearchResult = {
  name: "SearchResult",
  template: `
    <div class="row">
      <div class="cell"><img :src="thumbnail_url" height="48" width="48"></div>
      <div class="cell"><a :href="uri">{{ title }}</a></div>
      <div class="cell">{{ artistsLabel }}</div>
      <div class="cell">{{ collection }}</div>
    </div>
  `,
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

// TODO: display result in table
// TODO: "show more" ? More results in list ?
// TODO: lazy loading of images
// TODO: auto search with debounce ?
// TODO: improve styling
