import { computed, onMounted, reactive } from "vue";

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
      query: "",
      loading: false,
      error: null,
      data: null,
    });

    let abortController;

    const fetchResults = async () => {
      abortController?.abort();
      abortController = new AbortController();

      results.query = formData.query;
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
    <div class="mrgt+">
      <Label>
        <template v-if="loading">
          <Skeleton width="15em" />
        </template>
        <template v-else-if="error">Failed to retrieve results</template>
        <template v-else>
          {{ total }} results
          <template v-if="total > items.length"> ({{ items.length }} displayed)</template>
          <template v-if="query"> for <em>{{ query }}</em></template>
        </template>
      </Label>

      <div v-if="items.length || loading" class="mrgt+">
        <div class="row row--header">
          <div class="cell">TRACK</div>
          <div class="cell">ARTISTS</div>
          <div class="cell">COLLECTION</div>
        </div>

        <template v-if="loading">
          <SearchResultSkeleton v-for="n in 10"/>
        </template>
        <template v-else>
          <SearchResult v-for="item in items" v-bind="item"/>
        </template>
      </div>
    </div>
  `,
  props: {
    query: {
      type: String,
      required: true,
    },
    error: Error,
    loading: {
      type: Boolean,
      default: false,
    },
    data: Object,
  },
  setup(props) {
    const items = computed(() => props.data?.items ?? []);
    const total = computed(() => props.data?.total ?? 0);
    return { items, total };
  },
};

export const SearchResult = {
  template: `
    <div class="row">
      <div class="cell">
        <LazyImage :src="thumbnail_url" height="48" width="48" class="mrgr" />
        <a :href="uri">{{ title }}</a>
      </div>
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
        <Skeleton height="48px" width="48px" class="mrgr" />
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
      default: "1em",
    },
    width: {
      type: String,
      default: "4em",
    },
  },
};

// TODO: more columns
// TODO: show title and artist in the same column
// TODO: sort items by column
// TODO: submit button or debounce
// TODO: optimize results.data with shallowReactive ?
// TODO: links for playlist
