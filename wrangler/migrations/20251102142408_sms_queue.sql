CREATE OR REPLACE FUNCTION notifymsg() RETURNS trigger AS $$
DECLARE
BEGIN
  IF NEW.outgoing THEN
    PERFORM pg_notify('sent', CAST(NEW.id AS text));
  ELSE 
    PERFORM pg_notify('received', CAST(NEW.id AS text));
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER notify_msg AFTER INSERT ON messages FOR EACH ROW EXECUTE FUNCTION notifymsg();
