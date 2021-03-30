// TODO: Use prod module in "release" mode ?
import { createApp } from "/static/vendor/vue@3.0.9/vue.esm-browser.js";

export function createMainApp() {
  const app = createApp(App);
  app.component("Counter", Counter);
  app.component("Label", Label);
  return app;
}

export const App = {
  template: "#App",
};

export const Counter = {
  template: "#Counter",
  data() {
    return {
      counter: 0,
      items: [
        {
          title: "Sequence",
          artist: "Else",
          thumbnail:
            "https://i.scdn.co/image/ab67616d00004851a28752fcf5d966c3ef9e6f2d",
        },
        {
          title: "Sequence",
          artist: "Else",
          thumbnail:
            "https://i.scdn.co/image/ab67616d00004851a28752fcf5d966c3ef9e6f2d",
        },
        {
          title: "Sequence",
          artist: "Else",
          thumbnail:
            "https://i.scdn.co/image/ab67616d00004851a28752fcf5d966c3ef9e6f2d",
        },
      ],
    };
  },
};

export const Label = {
  template: `
    <strong><slot/></strong>
  `,
};
