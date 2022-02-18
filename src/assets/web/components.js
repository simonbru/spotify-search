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
  app.component("SearchResultSkeleton", SearchResultSkeleton);
  app.component("Skeleton", Skeleton);
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
    <SearchResults v-bind="results" />
  `,
  setup() {
    const formData = reactive({ query: "" });
    const results = reactive({
      loading: false,
      error: null,
      data: null,
    });

    const fetchResults = async () => {
      results.loading = true;
      results.error = null;

      try {
        const query = formData.query;
        // TODO: escape query
        const response = await fetch(`/api/search?q=${query}`);
        results.data = await response.json();
      } catch (err) {
        results.error = err;
        throw err;
      } finally {
        results.loading = false;
      }
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
        <template v-if="loading">
          <SearchResultSkeleton v-for="n in 10"/>
        </template>
        <template v-else-if="error">
          <em>Failed to retrieve results.</em>
        </template>
        <template v-else-if="data">
          <SearchResult v-for="item in data.items" v-bind="item"/>
        </template>
      </div>
    </div>
  `,
  props: {
    error: Error,
    loading: Boolean,
    data: Object,
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

export const SearchResultSkeleton = {
  name: "SearchResultSkeleton",
  template: `
    <div class="row">
      <div class="cell">
        <Skeleton height="48px" width="48px" />
      </div>
      <div class="cell">
        <Skeleton width="60%" />
      </div>
      <div class="cell">
        <Skeleton width="60%" />
      </div>
      <div class="cell">
        <Skeleton width="60%" />
      </div>
    </div>
  `,
};

export const Skeleton = {
  name: "Skeleton",
  template: `
    <div class="skeleton" :style="{ height, width }"></div>
  `,
  props: {
    height: {
      type: String,
      default: "16px", // default font size
    },
    width: {
      type: String,
      default: "100px",
    },
  },
};

// TODO: show link for track in playlist
// TODO: show more colums
// TODO: show title and artist in the same column
// TODO: "show more" ? More results in list ?
// TODO: sort items by column
// TODO: show links for playlist
// TODO: lazy loading of images
// TODO: cancel previous request
// TODO: improve styling
