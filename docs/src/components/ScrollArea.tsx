import { useEffect, useId, useRef, useState, type ReactNode } from 'react';

interface Props {
  children: ReactNode;
  height: string;
  style?: React.CSSProperties;
  viewportStyle?: React.CSSProperties;
  contentStyle?: React.CSSProperties;
  trackStyle?: React.CSSProperties;
  thumbStyle?: React.CSSProperties;
  dragToScroll?: boolean;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

export function ScrollArea({
  children,
  height,
  style,
  viewportStyle,
  contentStyle,
  trackStyle,
  thumbStyle,
  dragToScroll = false,
}: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const [thumbHeight, setThumbHeight] = useState(0);
  const [thumbTop, setThumbTop] = useState(0);
  const [isDragging, setIsDragging] = useState(false);
  const [isPanning, setIsPanning] = useState(false);
  const [isHovered, setIsHovered] = useState(false);
  const dragStartY = useRef(0);
  const dragStartScrollTop = useRef(0);
  const panStartX = useRef(0);
  const panStartY = useRef(0);
  const panStartScrollLeft = useRef(0);
  const panStartScrollTop = useRef(0);
  const viewportClassName = `scroll-area__viewport-${useId().replace(/:/g, '')}`;

  const updateScrollbar = () => {
    const container = containerRef.current;
    const content = contentRef.current;
    if (!container || !content) return;

    const containerHeight = container.clientHeight;
    const contentHeight = content.scrollHeight;

    if (contentHeight <= containerHeight) {
      setThumbHeight(0);
      setThumbTop(0);
      return;
    }

    const ratio = containerHeight / contentHeight;
    const nextThumbHeight = Math.max(ratio * containerHeight, 30);
    const scrollRange = contentHeight - containerHeight;
    const trackRange = containerHeight - nextThumbHeight;

    setThumbHeight(nextThumbHeight);
    setThumbTop(
      scrollRange > 0 ? (container.scrollTop / scrollRange) * trackRange : 0,
    );
  };

  useEffect(() => {
    updateScrollbar();
    const container = containerRef.current;
    const content = contentRef.current;
    if (!container) return;

    const handleScroll = () => updateScrollbar();
    container.addEventListener('scroll', handleScroll);

    const resizeObserver = new ResizeObserver(updateScrollbar);
    resizeObserver.observe(container);
    if (content) {
      resizeObserver.observe(content);
    }

    window.addEventListener('resize', updateScrollbar);

    return () => {
      container.removeEventListener('scroll', handleScroll);
      resizeObserver.disconnect();
      window.removeEventListener('resize', updateScrollbar);
    };
  }, [children]);

  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    setIsDragging(true);
    dragStartY.current = e.clientY;
    dragStartScrollTop.current = containerRef.current?.scrollTop || 0;
  };

  useEffect(() => {
    if (!isDragging) return;

    const handleMouseMove = (e: MouseEvent) => {
      const container = containerRef.current;
      const content = contentRef.current;
      if (!container || !content) return;

      const deltaY = e.clientY - dragStartY.current;
      const scrollRange = content.scrollHeight - container.clientHeight;
      const trackRange = container.clientHeight - thumbHeight;
      if (trackRange <= 0 || scrollRange <= 0) return;

      const ratio = scrollRange / trackRange;
      container.scrollTop = clamp(
        dragStartScrollTop.current + deltaY * ratio,
        0,
        scrollRange,
      );
    };

    const handleMouseUp = () => setIsDragging(false);

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isDragging, thumbHeight]);

  const handleViewportMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!dragToScroll || e.button !== 0) {
      return;
    }

    const target = e.target as HTMLElement;
    if (target.dataset.role === 'thumb') {
      return;
    }

    const container = containerRef.current;
    if (!container) return;

    setIsPanning(true);
    panStartX.current = e.clientX;
    panStartY.current = e.clientY;
    panStartScrollLeft.current = container.scrollLeft;
    panStartScrollTop.current = container.scrollTop;
    e.preventDefault();
  };

  useEffect(() => {
    if (!isPanning) return;

    const previousUserSelect = document.body.style.userSelect;
    const previousCursor = document.body.style.cursor;
    document.body.style.userSelect = 'none';
    document.body.style.cursor = 'grabbing';

    const handleMouseMove = (e: MouseEvent) => {
      const container = containerRef.current;
      if (!container) return;

      container.scrollLeft =
        panStartScrollLeft.current - (e.clientX - panStartX.current);
      container.scrollTop =
        panStartScrollTop.current - (e.clientY - panStartY.current);
    };

    const handleMouseUp = () => setIsPanning(false);

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.body.style.userSelect = previousUserSelect;
      document.body.style.cursor = previousCursor;
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isPanning]);

  const handleTrackClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if ((e.target as HTMLElement).dataset.role === 'thumb') {
      return;
    }

    const container = containerRef.current;
    const content = contentRef.current;
    if (!container || !content) return;

    const rect = e.currentTarget.getBoundingClientRect();
    const clickY = e.clientY - rect.top - thumbHeight / 2;
    const trackRange = container.clientHeight - thumbHeight;
    const scrollRange = content.scrollHeight - container.clientHeight;
    if (trackRange <= 0 || scrollRange <= 0) return;

    const ratio = clamp(clickY, 0, trackRange) / trackRange;
    container.scrollTop = ratio * scrollRange;
  };

  const showScrollbar = thumbHeight > 0;
  const trackOpacity = !showScrollbar ? 0 : isHovered || isDragging ? 1 : 0.56;

  return (
    <>
      <style>
        {`
          .${viewportClassName} {
            scrollbar-width: none;
            -ms-overflow-style: none;
          }

          .${viewportClassName}::-webkit-scrollbar {
            width: 0;
            height: 0;
            display: none;
          }
        `}
      </style>
      <div
        style={{
          position: 'relative',
          height: '100%',
          ...style,
        }}
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
      >
        <div
          ref={containerRef}
          className={viewportClassName}
          onMouseDown={handleViewportMouseDown}
          style={{
            height,
            overflow: 'auto',
            overscrollBehavior: 'contain',
            cursor: dragToScroll ? (isPanning ? 'grabbing' : 'grab') : 'auto',
            ...viewportStyle,
          }}
        >
          <div
            ref={contentRef}
            style={{
              minHeight: '100%',
              ...contentStyle,
            }}
          >
            {children}
          </div>
        </div>

        <div
          onClick={handleTrackClick}
          style={{
            position: 'absolute',
            top: 0,
            right: 0,
            width: '12px',
            height,
            opacity: trackOpacity,
            transition: 'opacity 160ms ease',
            pointerEvents: showScrollbar ? 'auto' : 'none',
            ...trackStyle,
          }}
        >
          <div
            data-role="thumb"
            onMouseDown={(e) => {
              e.stopPropagation();
              handleMouseDown(e);
            }}
            style={{
              position: 'absolute',
              top: `${thumbTop}px`,
              right: '3px',
              width: '6px',
              height: `${thumbHeight}px`,
              borderRadius: '999px',
              background: isDragging
                ? 'rgba(37, 99, 235, 0.65)'
                : 'rgba(100, 116, 139, 0.45)',
              boxShadow: isDragging
                ? '0 0 0 1px rgba(191, 219, 254, 0.8)'
                : 'none',
              cursor: isDragging ? 'grabbing' : 'grab',
              transition: isDragging
                ? 'none'
                : 'background 160ms ease, box-shadow 160ms ease',
              ...thumbStyle,
            }}
          />
        </div>
      </div>
    </>
  );
}
