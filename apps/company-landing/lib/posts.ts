import { readdir } from "fs/promises";

export interface Post {
  slug: string;
  title: string;
  categories: string[];
  publishedAt: Date;
  description: string;
}

export async function getPosts(): Promise<Post[]> {
  // Retrieve slugs from post routes
  const slugs = (
    await readdir("./app/blog/(posts)", {
      withFileTypes: true,
      recursive: true,
    })
  ).filter((dirent) => dirent.isDirectory() && !dirent.name.includes("@"));

  // Retrieve metadata from MDX files
  const posts = await Promise.all(
    slugs.map(async ({ name }) => {
      const { metadata } = await import(`../app/blog/(posts)/${name}/page.mdx`);
      return { slug: name, ...metadata };
    }),
  );

  // Sort posts from newest to oldest
  posts.sort((a, b) => +new Date(b.publishedAt) - +new Date(a.publishedAt));

  return posts;
}
