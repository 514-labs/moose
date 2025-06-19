import { Icons } from "./icons";
import { ContactCards, ContactCard } from "./contact-card";
import { paths } from "@/lib/paths";
import { Heading, HeadingLevel } from "./typography";

interface ContactProps {
  showHeading?: boolean;
}

export function Contact({ showHeading = true }: ContactProps) {
  return (
    <div className="space-y-6">
      {showHeading && (
        <div className="space-y-2">
          <Heading level={HeadingLevel.l3} className="text-primary">
            Learning & Community Resources
          </Heading>
          <p className="text-muted-foreground text-sm">
            Get help, learn, contribute, and stay connected with the Moose
            community
          </p>
        </div>
      )}

      <ContactCards columns={3}>
        <ContactCard
          title="Contribute"
          description="Check out the code, contribute to Moose, and report issues"
          ctaLink={paths.github}
          ctaLabel="Contribute"
          Icon={Icons.github}
        />
        <ContactCard
          title="Join Our Community"
          description="Connect with developers and get help with your projects"
          ctaLink={paths.slack}
          ctaLabel="Join Slack"
          Icon={Icons.slack}
        />
        <ContactCard
          title="Talk to Us"
          description="Contact the Moose maintainers for support and feedback"
          ctaLink={paths.calendly}
          ctaLabel="Schedule a Call"
          Icon={Icons.contact}
        />
        <ContactCard
          title="Learn & Watch"
          description="Video tutorials, demos, and deep-dives into Moose features"
          ctaLink={paths.youtube}
          ctaLabel="Watch Tutorials"
          Icon={Icons.youtube}
        />

        <ContactCard
          title="Follow Us on X"
          description="Follow us on X for the latest news and updates"
          ctaLink={paths.twitter}
          ctaLabel="Follow Us"
          Icon={Icons.twitter}
        />
      </ContactCards>
    </div>
  );
}
