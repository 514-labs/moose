import {
  APISource,
  isStreamResult,
  isSingleResult,
  APIContext,
} from "./apiSource";
import { expect } from "chai";

interface PokemonResource {
  name: string;
  url: string;
}

interface PokemonResponse {
  count: number;
  next: string | null;
  previous: string | null;
  results: PokemonResource[];
}

describe("APISource", function () {
  this.timeout(20000);

  it("should handle a single request (no pagination)", async () => {
    const singleSource = new APISource<PokemonResponse, PokemonResource[]>({
      name: "single-test",
      baseUrl: "https://pokeapi.co/api/v2",
      endpoint: "/pokemon",
      extractItems: (context: APIContext<PokemonResponse>) =>
        context.response.results,
    });
    const connectionTest = await singleSource.testConnection();
    expect(connectionTest.success).to.be.true;
    const result = await singleSource.extract();
    expect(Array.isArray(result)).to.be.true;
    if (Array.isArray(result)) {
      expect(result).to.not.be.empty;
      expect(result[0]).to.have.property("name");
      expect(result[0]).to.have.property("url");
    }
  });

  it("should paginate and yield items with async iteration", async () => {
    const pokemonSource = new APISource<PokemonResponse, PokemonResource[]>({
      name: "pokemon-test",
      baseUrl: "https://pokeapi.co/api/v2",
      endpoint: "/pokemon",
      extractItems: (context: APIContext<PokemonResponse>) =>
        context.response.results,
      pagination: {
        getNextUrl: (context: APIContext<PokemonResponse>) =>
          context.response.next,
        maxPages: 2,
        delayBetweenRequests: 100,
        retryConfig: { maxRetries: 2, backoffMs: 500 },
      },
      headers: {
        "User-Agent": "Mozilla/5.0 (compatible; MyApp/1.0)",
        Accept: "application/json",
        "X-Custom-Header": "custom-value",
        "Cache-Control": "no-cache",
      },
    });
    const connectionTest = await pokemonSource.testConnection();
    expect(connectionTest.success).to.be.true;
    let count = 0;
    for await (const pokemon of pokemonSource) {
      expect(pokemon).to.have.property("name");
      expect(pokemon).to.have.property("url");
      count++;
    }
    expect(count).to.be.greaterThan(0);
  });

  it("should return a stream from extract() for paginated APIs", async () => {
    const pokemonSource = new APISource<PokemonResponse, PokemonResource[]>({
      name: "pokemon-stream-test",
      baseUrl: "https://pokeapi.co/api/v2",
      endpoint: "/pokemon",
      extractItems: (context: APIContext<PokemonResponse>) =>
        context.response.results,
      pagination: {
        getNextUrl: (context: APIContext<PokemonResponse>) =>
          context.response.next,
        maxPages: 2,
        delayBetweenRequests: 100,
      },
    });
    const connectionTest = await pokemonSource.testConnection();
    expect(connectionTest.success).to.be.true;
    const result = await pokemonSource.extract();
    expect(isStreamResult(result)).to.be.true;
    if (isStreamResult(result)) {
      let count = 0;
      for await (const pokemon of result) {
        expect(pokemon).to.have.property("name");
        expect(pokemon).to.have.property("url");
        count++;
      }
      expect(count).to.be.greaterThan(0);
    }
  });
});
