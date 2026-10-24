package javax.microedition.midlet;

// Compile-only signatures; the fixture JAR uses the real emulator classes.
public abstract class MIDlet {
    protected abstract void startApp();
    protected abstract void pauseApp();
    protected abstract void destroyApp(boolean unconditional);
}
