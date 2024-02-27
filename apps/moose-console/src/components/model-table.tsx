import { DataModel } from "app/db";
import { Separator } from "./ui/separator";

const processType = (type: string | object) => {
  if (typeof type === "string") {
    return type;
  } else if (typeof type === "object" && type["Enum"] !== undefined) {
    return type["Enum"]["name"];
  }
  return JSON.stringify(type);
};

export default function ModelTable({ datamodel }: { datamodel: DataModel }) {
  return (
    <div>
      <div>
        <div className="flex py-4">
          <div className="grow basis-1">Field Name</div>
          <div className="grow basis-1"> Type</div>
          <div className="grow basis-1"> Required?</div>
          <div className="grow basis-1"> Unique?</div>
          <div className="grow basis-1"> Primary Key?</div>
        </div>
        <Separator />
      </div>
      {datamodel &&
        datamodel.columns.map((field, index) => (
          <div key={index}>
            <div className="flex py-4">
              <div className="grow basis-1 text-muted-foreground">
                {field.name}
              </div>
              <div className="grow basis-1 text-muted-foreground">
                {processType(field.data_type)}
              </div>
              <div className="grow basis-1 text-muted-foreground">
                {field.arity}
              </div>
              <div className="grow basis-1 text-muted-foreground">
                {`${field.unique}`}
              </div>
              <div className="grow basis-1 text-muted-foreground">
                {`${field.primary_key}`}
              </div>
            </div>
            {index !== datamodel.columns.length - 1 && <Separator />}
          </div>
        ))}
    </div>
  );
}
