import test.T;
import dep.*;
import org.junit.Test;

public class Main {
    public static void main(String[] args) {

        var t = new T();
        Library.foo = 4;
        // Library.foo = 4;
        System.out.println("Hello, World!");
    }
}
