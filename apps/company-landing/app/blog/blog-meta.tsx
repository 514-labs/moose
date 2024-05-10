import { Text } from "@514labs/design-system/typography";
import { humanReadableDate } from "../../lib/formatter";

interface BlogMetaProps {
  meta: any;
}

export default async function BlogMeta({ meta }: BlogMetaProps) {
  return (
    <div>
      <Text className="mb-0">
        {"Published " + humanReadableDate(meta.publishedAt)}
      </Text>
      {meta.categories &&
        meta.categories.map((cat: string, i: number) => {
          if (i === meta.categories.length - 1) {
            return (
              <Text className="mt-0 text-muted-foreground" key={i}>
                {cat}
              </Text>
            );
          } else {
            return (
              <Text className="mt-0 text-muted-foreground" key={i}>
                {cat},{" "}
              </Text>
            );
          }
        })}
      {meta.author && <Text> by {meta.author} </Text>}
    </div>
  );
}
