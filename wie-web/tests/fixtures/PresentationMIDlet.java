import javax.microedition.midlet.MIDlet;
import javax.microedition.lcdui.Canvas;
import javax.microedition.lcdui.Display;
import javax.microedition.lcdui.Graphics;

public final class PresentationMIDlet extends MIDlet {
    protected void startApp() {
        Output output = new Output();
        output.setFullScreenMode(true);
        Display.getDisplay(this).setCurrent(output);
    }

    protected void pauseApp() {}
    protected void destroyApp(boolean unconditional) {}

    private static final class Output extends Canvas {
        private boolean alternate;

        protected void keyPressed(int key) {
            alternate = !alternate;
            repaint();
        }

        protected void paint(Graphics graphics) {
            int[] colors = { alternate ? 0xff00ff : 0xff0000, 0x00ff00, 0x0000ff, 0x404040, 0x808080, 0xffffff };
            for (int i = 0; i < colors.length; i++) {
                graphics.setColor(colors[i]);
                graphics.fillRect((i % 3) * 80, (i / 3) * 160, 80, 160);
            }
            System.out.println(alternate ? "presentation-alternate" : "presentation-original");
        }
    }
}
