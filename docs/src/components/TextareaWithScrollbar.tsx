import { useState, useRef, useEffect } from 'react';

interface Props {
  value: string;
  onChange: (value: string) => void;
  style?: React.CSSProperties;
}

export function TextareaWithScrollbar({ value, onChange, style }: Props) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const [thumbHeight, setThumbHeight] = useState(0);
  const [thumbTop, setThumbTop] = useState(0);
  const [isDragging, setIsDragging] = useState(false);
  const [isHovered, setIsHovered] = useState(false);
  const dragStartY = useRef(0);
  const dragStartScrollTop = useRef(0);

  const updateScrollbar = () => {
    const textarea = textareaRef.current;
    if (!textarea) return;

    const { clientHeight, scrollHeight, scrollTop } = textarea;
    if (scrollHeight <= clientHeight) {
      setThumbHeight(0);
      return;
    }

    const ratio = clientHeight / scrollHeight;
    setThumbHeight(Math.max(ratio * clientHeight, 30));
    setThumbTop((scrollTop / scrollHeight) * clientHeight);
  };

  useEffect(() => {
    updateScrollbar();
  }, [value]);

  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    setIsDragging(true);
    dragStartY.current = e.clientY;
    dragStartScrollTop.current = textareaRef.current?.scrollTop || 0;
  };

  useEffect(() => {
    if (!isDragging) return;

    const handleMouseMove = (e: MouseEvent) => {
      const textarea = textareaRef.current;
      if (!textarea) return;

      const deltaY = e.clientY - dragStartY.current;
      const ratio = textarea.scrollHeight / textarea.clientHeight;
      textarea.scrollTop = dragStartScrollTop.current + deltaY * ratio;
      updateScrollbar();
    };

    const handleMouseUp = () => setIsDragging(false);

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isDragging]);

  const handleTrackClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const textarea = textareaRef.current;
    if (!textarea) return;

    const rect = e.currentTarget.getBoundingClientRect();
    const clickY = e.clientY - rect.top;
    const ratio = clickY / textarea.clientHeight;
    textarea.scrollTop = ratio * textarea.scrollHeight - textarea.clientHeight / 2;
    updateScrollbar();
  };

  const showScrollbar = thumbHeight > 0 && (isHovered || isDragging);

  return (
    <div
      style={{ position: 'relative', height: style?.height }}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      <textarea
        ref={textareaRef}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onScroll={updateScrollbar}
        style={{
          ...style,
          scrollbarWidth: 'none',
        }}
      />

      {/* 滚动条轨道 */}
      <div
        onClick={handleTrackClick}
        style={{
          position: 'absolute',
          top: 0,
          right: 0,
          width: '8px',
          height: '100%',
          opacity: showScrollbar ? 1 : 0,
          transition: 'opacity 0.2s',
        }}
      >
        <div
          onMouseDown={handleMouseDown}
          style={{
            position: 'absolute',
            right: '2px',
            width: '4px',
            height: `${thumbHeight}px`,
            top: `${thumbTop}px`,
            background: isDragging ? '#888' : '#bbb',
            borderRadius: '2px',
            cursor: 'pointer',
          }}
        />
      </div>
    </div>
  );
}
