import { SmallText } from "@/components/typography";
import React from "react";
import { Button } from "@/components/ui/button";

/**
 * Meta item definition for sidebar navigation
 */
interface MetaItem {
  type?: string;
  title?: string | React.ReactNode;
  theme?: Record<string, any>;
  display?: string;
  href?: string;
  Icon?: React.ElementType;
  newWindow?: boolean;
}

/**
 * Reusable JSX elements for consistent rendering
 */
const UI = {
  // Standard text with hover effect
  HoverText: ({ children }: { children: React.ReactNode }) => (
    <SmallText className="my-0 text-muted-foreground hover:text-primary transition-colors">
      {children}
    </SmallText>
  ),

  // Standard small text for navigation items
  NavText: ({ children }: { children: React.ReactNode }) => (
    <SmallText className="text-inherit transition-colors">{children}</SmallText>
  ),

  // Icon with text layout
  IconWithText: ({
    icon,
    text,
  }: {
    icon?: React.ReactElement;
    text: React.ReactNode;
  }) => (
    <div className="flex items-center gap-2 text-primary">
      {icon}
      <SmallText className="text-inherit my-2">{text}</SmallText>
    </div>
  ),
};

/**
 * Renders a string item as a paragraph with styles
 */
const renderStringItem = (item: string): MetaItem => {
  return {
    title: <UI.HoverText>{item}</UI.HoverText>,
  };
};

/**
 * Renders a page type item
 */
const renderPageItem = (item: MetaItem): MetaItem => {
  return {
    ...item,
    title: <UI.NavText>{item.title}</UI.NavText>,
  };
};

/**
 * Renders an object item with icon
 */
const renderObjectWithIcon = (item: MetaItem): MetaItem => {
  return {
    ...item,
    title: (
      <UI.IconWithText
        icon={item.Icon ? <item.Icon className="w-6 h-6" /> : undefined}
        text={item.title}
      />
    ),
  };
};

/**
 * Renders a standard object item
 */
const renderStandardObject = (item: MetaItem): MetaItem => {
  // If title is already a React element, return as is
  if (React.isValidElement(item.title)) {
    return item;
  }

  // Otherwise, wrap the title in our component
  return {
    ...item,
    title: <UI.HoverText>{item.title}</UI.HoverText>,
  };
};

/**
 * Renders an index item with theme
 */
const renderIndexWithTheme = (item: MetaItem): MetaItem => {
  const processedItem = { ...item };

  if (typeof item.title === "string") {
    processedItem.title = <UI.IconWithText text={item.title} />;
  }

  return processedItem;
};

/**
 * Renders an item with display or theme properties
 */
const renderDisplayOrTheme = (item: MetaItem): MetaItem => {
  const processedItem = { ...item };

  if (typeof item.title === "string") {
    processedItem.title = <UI.HoverText>{item.title}</UI.HoverText>;
  }

  return processedItem;
};

/**
 * Determines the type of item and delegates to the appropriate renderer
 */
const renderItem = (item: string | MetaItem, key?: string): MetaItem => {
  // Handle string item
  if (typeof item === "string") {
    return renderStringItem(item);
  }

  // Handle page type
  if (item.type === "page") {
    return renderPageItem(item);
  }

  // Handle object with non-React element title
  if (typeof item === "object" && !React.isValidElement(item.title)) {
    return renderObjectWithIcon(item);
  }

  // Handle object with title
  if (item.title) {
    return renderStandardObject(item);
  }

  // Default case
  return item;
};

/**
 * Process the meta object structure using specialized renderers
 */
export const render = (items: Record<string, any>): Record<string, any> => {
  const result: Record<string, any> = {};

  for (const key in items) {
    const item = items[key];

    // Skip undefined or null items
    if (!item) continue;

    // Determine the type of item and render appropriately
    if (typeof item === "object" && !React.isValidElement(item)) {
      // Special case: index item with theme
      if (key === "index" && item.theme) {
        result[key] = renderIndexWithTheme(item);
      }
      // Items with display or theme properties
      else if (item.display || item.theme) {
        result[key] = renderDisplayOrTheme(item);
      }
      // Standard object items
      else {
        result[key] = renderItem(item, key);
      }
    }
    // String or React element items
    else {
      result[key] = renderItem(item, key);
    }
  }

  return result;
};
