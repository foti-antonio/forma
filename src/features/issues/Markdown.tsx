import Markdown from 'react-markdown';
export function MarkdownPreview({ text }: { text: string }) {
  return (
    <div className="markdown-preview">
      <Markdown
        skipHtml
        components={{
          // Links remain readable without navigating the desktop WebView or loading remote assets.
          a: ({ children }) => (
            <span className="markdown-link">{children}</span>
          ),
          img: ({ alt }) => (
            <span className="muted">[Image: {alt || 'attachment'}]</span>
          ),
        }}
      >
        {text || '*No description yet.*'}
      </Markdown>
    </div>
  );
}
