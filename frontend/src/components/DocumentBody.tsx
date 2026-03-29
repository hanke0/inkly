import { useMemo } from "react";

import { buildDocumentBodyRender } from "../lib/documentContent";

type DocumentBodyProps = {
  content: string;
};

export function DocumentBody({ content }: DocumentBodyProps) {
  const render = useMemo(() => buildDocumentBodyRender(content), [content]);

  if (render.kind === "html") {
    return (
      <div className="inkly-reading__iframe-shell mt-6 min-w-0 max-w-full">
        <iframe
          title="Document HTML"
          className="inkly-reading__iframe"
          srcDoc={render.srcdoc}
          sandbox=""
          referrerPolicy="no-referrer"
        />
      </div>
    );
  }

  return (
    <div
      className="inkly-reading__body inkly-reading__body--rich mt-6"
      dangerouslySetInnerHTML={{ __html: render.html }}
    />
  );
}
