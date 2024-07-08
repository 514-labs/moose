import { Header } from "@tanstack/react-table";

export const ColumnResizer = ({ header }: { header: Header<any, unknown> }) => {
  if (header.column.getCanResize() === false) return <></>;

  return (
    <div
      {...{
        onMouseDown: (e: any) => {
          console.log(header.getResizeHandler(e));
          header.getResizeHandler(e);
        },
        onTouchStart: header.getResizeHandler(),
        className: `absolute top-0 right-0 cursor-col-resize w-px h-full bg-gray-900 hover:bg-gray-700 hover:w-2`,
        style: {
          userSelect: "none",
          touchAction: "none",
        },
      }}
    />
  );
};
