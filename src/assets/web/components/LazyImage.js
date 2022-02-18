import {
  inject,
  onMounted,
  ref,
} from "/static/vendor/vue@3.2.31/vue.esm-browser.js";

const lazyImageKey = Symbol("lazyImageKey");

export function installLazyImage(app) {
  const imageObserver = new IntersectionObserver(function (entries) {
    entries.forEach(function (entry) {
      if (entry.isIntersecting) {
        var image = entry.target;
        image.src = image.dataset.src;
        imageObserver.unobserve(image);
      }
    });
  });

  app.provide(lazyImageKey, imageObserver);
  app.component("LazyImage", LazyImage);
}

const LazyImage = {
  template: `
    <img v-bind="$attrs" :data-src="src" ref="img">
  `,
  props: {
    src: {
      type: String,
      required: true,
    },
  },
  setup() {
    const img = ref();
    const imageObserver = inject(lazyImageKey);

    onMounted(() => {
      imageObserver.observe(img.value);
    });

    return { img };
  },
};
export default LazyImage;
