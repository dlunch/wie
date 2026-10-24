package javax.microedition.lcdui;

public abstract class Canvas extends Displayable {
    public void setFullScreenMode(boolean fullscreen) {}
    public void repaint() {}
    protected abstract void paint(Graphics graphics);
}
