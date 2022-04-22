#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

struct BrokerOptions;

struct CallOptions;

struct Context;

struct HandlerResult;

struct Payload;

struct Schema;

struct SchemaActions;

struct Service;

struct ServiceBrokerMessage;

using BrokerSender = UnboundedSender<ServiceBrokerMessage>;

extern "C" {

Service *service_create(const Schema *schema, const BrokerSender *broker_sender);

void service_destroy(Service *svc);

void service_start(const Service *svc);

Schema *schema_create(const char *name, const char *version, SchemaActions *action);

void schema_destroy(Schema *schema);

SchemaActions *schema_action_create(const char *name, HandlerResult (*handler)(Context));

void schema_action_destroy(SchemaActions *action);

Context *context_create(const BrokerSender *broker_sender,
                        const char *node_id,
                        const char *service);

void context_destroy(Context *context);

void context_call(const Context *context,
                  const char *action_name,
                  const BrokerOptions *broker_options,
                  const Payload *params,
                  CallOptions *opts);

void context_emit(const Context *context, const char *event_name, Value *data, Value *opts);

void context_broadcast(const Context *context, const char *event_name, Value *data, Value *opts);

} // extern "C"
