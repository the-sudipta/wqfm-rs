// Prints the iteration order of a HashMap filled with keys of the given hash
// codes (one case per line), then "|", then the order of new HashMap<>(map).
// Keys are not Comparable, like wQFM's Quartet. Used with
// examples/java_order_check.rs to check wqfm-rs's model of HashMap order.
//
//   javac JavaOrderCheck.java && java JavaOrderCheck cases.txt > java.txt

import java.io.BufferedReader;
import java.io.FileReader;
import java.util.HashMap;
import java.util.Map;
import java.util.StringJoiner;

public class JavaOrderCheck {
    static final class Key {
        final int id;
        final int hash;

        Key(int id, int hash) {
            this.id = id;
            this.hash = hash;
        }

        @Override
        public int hashCode() {
            return hash;
        }

        @Override
        public boolean equals(Object o) {
            return o instanceof Key && ((Key) o).id == id;
        }
    }

    static String order(Map<Key, Integer> map) {
        StringJoiner j = new StringJoiner(" ");
        for (Key k : map.keySet()) {
            j.add(Integer.toString(k.id));
        }
        return j.toString();
    }

    public static void main(String[] args) throws Exception {
        StringBuilder out = new StringBuilder();
        try (BufferedReader r = new BufferedReader(new FileReader(args[0]))) {
            String line;
            while ((line = r.readLine()) != null) {
                String[] parts = line.trim().split(" ");
                Map<Key, Integer> map = new HashMap<>();
                for (int i = 0; i < parts.length; i++) {
                    map.put(new Key(i, Integer.parseInt(parts[i])), i);
                }
                Map<Key, Integer> copy = new HashMap<>(map);
                out.append(order(map)).append(" | ").append(order(copy)).append('\n');
            }
        }
        System.out.print(out);
    }
}
