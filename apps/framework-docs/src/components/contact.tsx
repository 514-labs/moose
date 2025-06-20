import { Icons } from "./icons";
import { ContactCards, ContactCard } from "./contact-card";
import { paths } from "@/lib/paths";
import React from "react";

interface ContactProps {
  variant?: "moose" | "aurora";
}

export function Contact({ variant = "moose" }: ContactProps) {
  return (
    <ContactCards columns={3}>
      {React.Children.map(
        [
          <ContactCard
            title="Contribute"
            description="Check out the code, contribute to Moose, and report issues"
            ctaLink={paths.github}
            ctaLabel="Contribute"
            Icon={Icons.github}
          />,
          <ContactCard
            title="Join Our Community"
            description="Connect with developers and get help with your projects"
            ctaLink={paths.slack}
            ctaLabel="Join Slack"
            Icon={Icons.slack}
          />,
          <ContactCard
            title="Talk to Us"
            description="Contact the Moose maintainers for support and feedback"
            ctaLink={paths.calendly}
            ctaLabel="Schedule a Call"
            Icon={Icons.contact}
          />,
          <ContactCard
            title="Learn & Watch"
            description="Video tutorials, demos, and deep-dives into Moose features"
            ctaLink={paths.youtube}
            ctaLabel="Watch Tutorials"
            Icon={Icons.youtube}
          />,

          <ContactCard
            title="Follow Us on X"
            description="Follow us on X for the latest news and updates"
            ctaLink={paths.twitter}
            ctaLabel="Follow Us"
            Icon={Icons.twitter}
          />,
        ],
        (child) => React.cloneElement(child, { variant }),
      )}
    </ContactCards>
  );
}
