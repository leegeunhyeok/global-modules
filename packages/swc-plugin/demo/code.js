export default class DOMRect extends DOMRectReadOnly {
  get x(): number {
    return this.__getInternalX();
  }

  set x(x?: number) {
    this.__setInternalX(x);
  }

  get y(): number {
    return this.__getInternalY();
  }

  set y(y?: number) {
    this.__setInternalY(y);
  }

  get width(): number {
    return this.__getInternalWidth();
  }

  set width(width?: number) {
    this.__setInternalWidth(width);
  }

  get height(): number {
    return this.__getInternalHeight();
  }

  set height(height?: number) {
    this.__setInternalHeight(height);
  }

  static fromRect(rect?: DOMRectInit): DOMRect {
    if (!rect) {
      return new DOMRect();
    }

    return new DOMRect(rect.x, rect.y, rect.width, rect.height);
  }
}
