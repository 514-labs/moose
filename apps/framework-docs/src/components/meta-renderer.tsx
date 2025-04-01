import { SmallText } from "@/components/typography";
import React from "react";
import { Button } from "@/components/ui/button";

// Define types for our meta items
type MetaItem = {
  type?: string;
  title?: string | React.ReactNode;
  theme?: Record<string, any>;
  display?: string;
  href?: string;
};

// Universal renderer that handles all sidebar item types
const renderSidebarItem = (item: string | MetaItem, key?: string): MetaItem => {
  // If it's a simple string, it's a regular menu item with the string as title
  if (typeof item === "string") {
    return {
      title: (
        <p className="my-0 text-muted-foreground hover:text-primary transition-colors">
          {item}
        </p>
      ),
    };
  }
  // If it's a page, it's a page item with the title and href
  if (item.type === "page") {
    return {
      type: "page",
      title: (
        <SmallText className="text-inherit transition-colors">
          {item.title}
        </SmallText>
      ),
      href: item.href,
    };
  }

  // If it has a type field, it's a separator
  if (item.type === "separator" && !item.title) {
    return {
      type: "separator",
      title: (
        <p className="text-sm mb-0">
          {typeof item.title === "string"
            ? item.title
            : key?.replace(/--/g, "")}
        </p>
      ),
    };
  }

  // If it's already an object with a title field
  if (item.title) {
    // If title is already a React element, return as is
    if (React.isValidElement(item.title)) {
      console.log("item.title", item.title);
      return item;
    }

    // Otherwise, wrap the title in our SmallText component
    return {
      ...item,
      title: (
        <p className="my-0 text-muted-foreground hover:text-primary transition-colors">
          {item.title}
        </p>
      ),
    };
  }

  // Default case - return the item as is
  return item;
};

// Process the meta object structure using our universal renderer
export const render = (items: Record<string, any>): Record<string, any> => {
  const result: Record<string, any> = {};

  for (const key in items) {
    const item = items[key];

    // Process the item based on its type
    if (typeof item === "object" && !React.isValidElement(item)) {
      // For objects that don't have the expected structure or are complex
      if (item.display || item.theme) {
        const processedItem = { ...item };
        if (typeof item.title === "string") {
          processedItem.title = renderSidebarItem(item.title).title;
        }
        result[key] = processedItem;
      } else {
        // It's an item with properties we should process
        result[key] = renderSidebarItem(item, key);
      }
    } else {
      // It's a simple item (string or React element)
      result[key] = renderSidebarItem(item, key);
    }
  }

  return result;
};
