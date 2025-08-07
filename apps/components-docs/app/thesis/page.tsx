function Thesis() {
  return (
    <div className="flex flex-col gap-4 max-w-2xl mx-auto">
      <div className="prose dark:prose-invert prose-neutral">
        <h1 className="text-4xl">Our thesis</h1>
        <p>
          Connectors are the building blocks to all data-intensive applications.
          Everyone hates creating them and the only thing worst than creating
          them is maintaining them.
        </p>
        <p>
          Connector providers have taken advantage of this by creating opaque,
          expensive connectors that you pay for but never actually own.
        </p>
        <p>It&apos;s time to change that.</p>
        <p>What the connector factory is:</p>
        <ul>
          <li>
            rock solid set of absractions and connector implementation patterns
          </li>
          <li>
            prompts and tools to build & test connectors &quot;yourself&quot;
          </li>
          <li>
            tools to distribute those connectors, integrate them directly in
            your applications and share them with your friends
          </li>
        </ul>
      </div>
    </div>
  );
}

export default Thesis;
