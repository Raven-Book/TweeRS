import { useState, useRef, useEffect, ReactNode } from 'react';

interface Props {
  children: ReactNode;
  height: string;
  style?: React.CSSProperties;
}

export function ScrollArea({ children, height, style }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const [thumbHeight, setThumbHeight] = useState(0);
  const [thumbTop, setThumbTop] = useState(0);
  const [isDragging, setIsDragging] = useState(false);
  const [isHovered, setIsHovered] = useState(false);
  const dragStartY = useRef(0);
  const dragStartScrollTop = useRef(0);

  // 计算滚动条尺寸
  const updateScrollbar = () => {
    const container = containerRef.current;
    const content = contentRef.current;
    if (!container || !content) return;

    const containerHeight = container.clientHeight;
    const contentHeight = content.scrollHeight;

    if (contentHeight <= containerHeight) {
      setThumbHeight(0);
      return;
    }

    const ratio = containerHeight / contentHeight;
    setThumbHeight(Math.max(ratio * containerHeight, 30));
    setThumbTop((container.scrollTop / contentHeight) * containerHeight);
  };

  useEffect(() => {
    updateScrollbar();
    const container = containerRef.current;
    if (!container) return;

    const handleScroll = () => updateScrollbar();
    container.addEventListener('scroll', handleScroll);

    const resizeObserver = new ResizeObserver(updateScrollbar);
    resizeObserver.observe(container);

    return () => {
      container.removeEventListener('scroll', handleScroll);
      resizeObserver.disconnect();
    };
  }, [children]);

  // 拖拽开始
  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    setIsDragging(true);
    dragStartY.current = e.clientY;
    dragStartScrollTop.current = containerRef.current?.scrollTop || 0;
  };

  // 拖拽移动
  useEffect(() => {
    if (!isDragging) return;

    const handleMouseMove = (e: MouseEvent) => {
      const container = containerRef.current;
      const content = contentRef.current;
      if (!container || !content) return;

      const deltaY = e.clientY - dragStartY.current;
      const ratio = content.scrollHeight / container.clientHeight;
      container.scrollTop = dragStartScrollTop.current + deltaY * ratio;
    };

    const handleMouseUp = () => setIsDragging(false);

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isDragging]);

  // 点击轨道跳转
  const handleTrackClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const container = containerRef.current;
    const content = contentRef.current;
    if (!container || !content) return;

    const rect = e.currentTarget.getBoundingClientRect();
    const clickY = e.clientY - rect.top;
    const ratio = clickY / container.clientHeight;
    container.scrollTop = ratio * content.scrollHeight - container.clientHeight / 2;
  };

  const showScrollbar = thumbHeight > 0 && (isHovered || isDragging);

  return (
    <div
      style={{
        position: 'relative',
        height,
        ...style,
      }}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      {/* 内容区域 */}
      <div
        ref={containerRef}
        style={{
          height: '100%',
          overflow: 'auto',
          scrollbarWidth: 'none',
        }}
      >
        <div ref={contentRef}>{children}</div>
      </div>

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
        {/* 滑块 */}
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
            transition: isDragging ? 'none' : 'background 0.2s',
          }}
        />
      </div>
    </div>
  );
}
