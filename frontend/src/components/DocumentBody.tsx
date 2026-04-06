import { useMemo } from 'react';

import { buildDocumentBodyRender } from '../lib/documentContent';
import { useI18n } from '../i18n/context';

type DocumentBodyProps = {
  content: string;
};

export function DocumentBody({ content }: DocumentBodyProps) {
  const { t } = useI18n();
  const render = useMemo(() => buildDocumentBodyRender(content), [content]);

  if (render.kind === 'html') {
    return (
      <div className="inkly-reading__iframe-root mt-6 flex min-h-0 min-w-0 flex-1 flex-col">
        <div className="inkly-reading__iframe-shell flex min-h-0 min-w-0 flex-1 flex-col">
          <iframe
            title={t('iframe.documentHtml')}
            className="inkly-reading__iframe min-h-0 w-full flex-1 border-0 bg-white"
            srcDoc={render.srcdoc}
            sandbox=""
            referrerPolicy="no-referrer"
          />
        </div>
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
