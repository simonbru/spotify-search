// TODO: Use prod module in "release" mode ?
import {
  onMounted,
  reactive,
} from "/static/vendor/vue@3.2.31/vue.esm-browser.js";

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

    let abortController;

    const fetchResults = async () => {
      abortController?.abort();
      abortController = new AbortController();
      results.loading = true;
      results.error = null;

      try {
        const params = new URLSearchParams({ q: formData.query });
        const response = await fetch(`/api/search?${params}`, {
          signal: abortController.signal,
        });
        results.data = await response.json();
        results.loading = false;
      } catch (err) {
        if (err.name === "AbortError") {
          return;
        }
        results.error = err;
        results.loading = false;
        throw err;
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
  template: `
    <div class="row">
      <div class="cell">
        <LazyImage :src="thumbnail_url" height="48" width="48" />
      </div>
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

// TODO: link for track in playlist
// TODO: more columns
// TODO: show title and artist in the same column
// TODO: "show more" ? More results in list ?
// TODO: sort items by column
// TODO: optimize results.data with shallowReactive ?
// TODO: links for playlist
// TODO: improve styling
