import { search } from "virtual:ox-content/search";

const results = await search("install");

for (const result of results) {
  console.log(result.title, result.url);
}
