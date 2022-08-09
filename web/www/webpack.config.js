const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
  entry: "./bootstrap.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js",
  },
  mode: "development",
  plugins: [
    new CopyWebpackPlugin(['index.html', 'ray_cache.txt', 'fixed_distance_distance_cache.txt', 'constellations.jpg', 'stars.jpg',
      'galaxy.jpg', 'fixed_distance_distance_cache64_64.txt', 'distance_cache16_256_64.txt', 'distance_cache16_64_64.txt', 'direction_cache.txt'])
  ],
};
