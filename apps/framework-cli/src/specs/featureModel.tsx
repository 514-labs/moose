export type Feature = {
  _id: string;
  _type: "feature";
  _createdAt: string;
  _updatedAt: string;
  _rev: string;
  slug?: string; // URL slug for the feature
  productNameRef?: string; // Reference to the product that this feature belongs to. i.e. Moose, Boreal or Aurora
  hero?: string; // Strong title of the feature (needs to stand on it's own)
  subHero?: string; // Short description of the feature (displayed in conjunction with the hero)
  subfeature?: Array<{
    subfeatureNameRef: string;
  }>; // A list of subfeatures that belong to this feature
  heading?: string; // Piffy title of the feature (needs to stand on it's own)
  subheading?: string; // Second part of the piffy title to enhance the preview point (can be empty)
  text?: string; // Description of the feature. Can be a couple sentences
  imgAlt?: string; // Description of the image for highlighting the feature (to be used by designers to actually create the graphic)
  headlines?: Array<{
    heading?: string;
    subheading?: string;
    _key: string;
  }>; // List of the big call outs for the feature. These should be self contained statements
  badge?: {
    text?: string;
    href?: string;
    variant?: "default" | "secondary" | "outline";
  }; // Status of the feature
  ctas?: Array<{
    label: string;
    href?: string;
    quickStartScript?: string;
  }>; // List of actions we want to display for the feature like reading the docs or getting started commands
  bulletPoints?: Array<string>; // List of value propositions or capabilitiesfor the feature
  label?: {
    text?: string; // The name of the feature
    icon?: string; // The icon name from lucide
  };
};

export type Subfeature = {
  _id: string;
  _type: "subfeature";
  _createdAt: string;
  _updatedAt: string;
  _rev: string;
  slug?: string; // URL slug for the subfeature
  featureNameRef?: string; // Reference to the feature that this subfeature belongs to. i.e. Moose, Boreal or Aurora
  hero?: string; // Strong title of the subfeature (needs to stand on it's own)
  subHero?: string; // Short description of the subfeature (displayed in conjunction with the hero)
  heading?: string; // Piffy title of the subfeature (needs to stand on it's own)
  subheading?: string; // Second part of the piffy title to enhance the preview point (can be empty)
  text?: string; // Description of the subfeature. Can be a couple sentences
  imgAlt?: string; // Description of the image for highlighting the subfeature (to be used by designers to actually create the graphic)
  headlines?: Array<{
    heading?: string;
    subheading?: string;
    _key: string;
  }>; // List of the big call outs for the subfeature. These should be self contained statements
  badge?: {
    text?: string;
    href?: string;
    variant?: "default" | "secondary" | "outline";
  }; // Status of the subfeature
  ctas?: Array<{
    label: string;
    href?: string;
    quickStartScript?: string;
  }>; // List of actions we want to display for the feature like reading the docs or getting started commands
  bulletPoints?: Array<string>; // List of value propositions or capabilitiesfor the feature
  label?: {
    text?: string; // The name of the feature
    icon?: string; // The icon name from lucide
  };
};
