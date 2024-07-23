export default function LanguageBadge() {
  return (
    <div
      className="border-2 border-transparent p-1 rounded-3xl"
      style={{
        borderImageSource:
          "linear-gradient(103.28deg, #641BFF -59.83%, #1983FF 200.23%, #C8FF2C 264.11%)",
        borderImageSlice: 1,
      }}
    >
      <span
        className="text-clip bg-primary text-transparent bg-clip-text"
        style={{
          backgroundImage:
            "linear-gradient(103.28deg, #641BFF -59.83%, #1983FF 200.23%, #C8FF2C 264.11%)",
        }}
      >
        JS
      </span>
    </div>
  );
}
