interface CodeBlockProps {
  code: string;
  language?: string;
  maxHeight?: string;
}

function CodeBlock({ code, language = 'text', maxHeight = '300px' }: CodeBlockProps) {
  return (
    <div className="code-block" style={{ maxHeight }}>
      <div className="code-block-header">
        <span className="code-block-language">{language}</span>
        <button className="btn btn-secondary" onClick={() => navigator.clipboard.writeText(code)}>
          复制
        </button>
      </div>
      <pre className="code-block-content">
        <code>{code}</code>
      </pre>
    </div>
  );
}

export default CodeBlock;