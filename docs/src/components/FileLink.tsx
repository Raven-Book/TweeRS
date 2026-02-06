interface Props {
  src: string;
  children?: React.ReactNode;
  alt?: string;
  download?: boolean;
}

const IMAGE_EXTS = /\.(png|jpe?g|gif|svg|webp|avif|ico|bmp)$/i;

function resolveBase(src: string): string {
  const base = import.meta.env.DEV ? '/' : '/TweeRS/';
  // src already starts with '/', strip it to avoid double slash
  return base + src.replace(/^\//, '');
}

export function FileLink({ src, children, alt, download }: Props) {
  const url = resolveBase(src);

  if (!download && IMAGE_EXTS.test(src)) {
    return <img src={url} alt={alt ?? src} />;
  }

  const filename = src.split('/').pop() ?? src;
  return (
    <a
      href={url}
      download={filename}
      style={{ color: '#07f', textDecoration: 'none', fontWeight: 500 }}
      onMouseEnter={(e) => (e.currentTarget.style.textDecoration = 'underline')}
      onMouseLeave={(e) => (e.currentTarget.style.textDecoration = 'none')}
    >
      {children ?? filename}
    </a>
  );
}
